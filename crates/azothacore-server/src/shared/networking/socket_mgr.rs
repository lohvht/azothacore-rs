use std::{fmt::Debug, hash::Hash, marker::PhantomData, sync::Arc};

use azothacore_common::{
    bevy_app::{az_startup_succeeded, AzStartupFailedEvent, TokioRuntime},
    configuration::ConfigMgr,
};
use bevy::{
    app::AppExit,
    ecs::world::CommandQueue,
    prelude::{App, Commands, Component, EventReader, EventWriter, IntoSystemConfigs, PostUpdate, Query, Res, Resource, Startup, SystemSet, Update},
    tasks::poll_once,
};
use tokio::{
    net::{TcpListener, TcpStream, ToSocketAddrs},
    sync::{
        mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
        OwnedSemaphorePermit,
        Semaphore,
    },
    task::JoinHandle,
};
use tracing::{debug, error};

use crate::shared::networking::socket::AddressOrName;

pub trait SocketMgrConfig<S>: Send + Sync + 'static {
    fn retrieve_bind_addr(&self) -> impl ToSocketAddrs;
    fn retrieve_max_connections(&self) -> Option<usize> {
        None
    }
}

/// Marker component trait for [socket_mgr_plugin] to identify bevy queries for
/// connection related entities.
pub trait ConnectionComponent: Component {
    fn connection_type() -> &'static str {
        std::any::type_name::<Self>()
    }
}

pub struct NewTcpConnection {
    pub permit: Option<OwnedSemaphorePermit>,
    pub name:   AddressOrName,
    pub conn:   TcpStream,
}

#[derive(SystemSet)]
pub struct SocketMgrStartNetworkSet<C, S>(PhantomData<(C, S)>);

impl<C, S> Default for SocketMgrStartNetworkSet<C, S> {
    fn default() -> Self {
        Self(PhantomData::<(C, S)>)
    }
}

impl<C, S> Clone for SocketMgrStartNetworkSet<C, S> {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}
impl<C, S> PartialEq for SocketMgrStartNetworkSet<C, S> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<C, S> Debug for SocketMgrStartNetworkSet<C, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ctype = std::any::type_name::<C>();
        let stype = std::any::type_name::<S>();
        f.debug_tuple("SocketMgrStartNetworkSet").field(&ctype).field(&stype).finish()
    }
}

impl<C, S> Hash for SocketMgrStartNetworkSet<C, S> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
    }
}

impl<C, S> Eq for SocketMgrStartNetworkSet<C, S> {}

#[derive(SystemSet)]
pub enum SocketMgrSet<S> {
    HandleNewSocket(PhantomData<S>),
    HandleReceivedSocket(PhantomData<S>),
    NetworkTermination(PhantomData<S>),
}

impl<S> SocketMgrSet<S> {
    pub fn handle_new_sockets() -> Self {
        Self::HandleNewSocket(PhantomData::<S>)
    }

    pub fn handle_received_socket() -> Self {
        Self::HandleReceivedSocket(PhantomData::<S>)
    }

    pub fn network_termination() -> Self {
        Self::NetworkTermination(PhantomData::<S>)
    }
}

impl<S> Clone for SocketMgrSet<S> {
    fn clone(&self) -> Self {
        match self {
            Self::HandleNewSocket(p) => Self::HandleNewSocket(*p),
            Self::HandleReceivedSocket(p) => Self::HandleReceivedSocket(*p),
            Self::NetworkTermination(p) => Self::NetworkTermination(*p),
        }
    }
}

impl<S> PartialEq for SocketMgrSet<S> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::HandleNewSocket(l0), Self::HandleNewSocket(r0)) => l0 == r0,
            (Self::HandleReceivedSocket(l0), Self::HandleReceivedSocket(r0)) => l0 == r0,
            (Self::NetworkTermination(l0), Self::NetworkTermination(r0)) => l0 == r0,
            _ => false,
        }
    }
}

impl<S> Debug for SocketMgrSet<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let stype = std::any::type_name::<S>();
        match self {
            Self::HandleNewSocket(_) => f.debug_tuple("HandleNewSocket").field(&stype).finish(),
            Self::HandleReceivedSocket(_) => f.debug_tuple("HandleReceivedSocket").field(&stype).finish(),
            Self::NetworkTermination(_) => f.debug_tuple("NetworkTermination").field(&stype).finish(),
        }
    }
}

impl<S> Hash for SocketMgrSet<S> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
    }
}

impl<S> Eq for SocketMgrSet<S> {}

/// [socket_mgr_plugin] is a bevy plugin that begins a TCP listener that asynchronously
/// accepts connections independent of bevy's runtime.
///
/// It requires a [azothacore_common::configuration::ConfigMgr] resource with a generic that
/// implements [SocketMgrConfig] which will be used to
/// init the network from config on the [`Startup`] schedule
///
/// New sockets are *NOT* handled for you, this is purely by design as some systems may have additional
/// startup requirements such as requiring other resources.
/// You should create a system to handle this. The system should take in
/// a [ResMut] of [SocketReceiver] to receive the socket and process it, then create the related
/// [ConnectionComponent] while removing the component [RunStartTcpSocketTask] of the same [ConnectionComponent].
///
/// For example, the below code snippet highlights an example of such a system, where
/// [MySocket] implements [ConnectionComponent] for a given [socket_mgr_plugin].
///
/// After which [receive_spawned_sockets] will take over and insert the [MySocket] component
/// ```
/// fn handle_and_spawn_received_sockets(mut commands: Commands, rt: Res<TokioRuntime>, mut sock_recv: ResMut<SocketReceiver<MySocket>>) {
///     while let Ok(sock) = sock_recv.0.try_recv() {
///         let entity = commands.spawn_empty().id();
///         let task = rt.spawn(async move {
///             let mut command_queue = CommandQueue::default();
///             let new_sock = match MySocket::start(sock).await {
///                 Err(e) => {
///                     error!(cause=?e, "error starting from new TCP connection");
///                     return command_queue;
///                 },
///                 Ok(s) => s,
///             };
///             command_queue.push(move |world: &mut World| {
///                 world.entity_mut(entity).insert(new_sock).remove::<RunStartTcpSocketTask<S>>();
///             });/
///             command_queue
///         });
///         commands.entity(entity).insert(RunStartTcpSocketTask::<S>::new(task));
///     }
/// }
/// ```
/// It listens to the [AppExit] event in the [PostUpdate] stage
///
/// The systems from this plugin can be accessed from [SocketMgrStartNetworkSet]
/// and [SocketMgrSet] and thus re-ordered if needed.
/// [SocketMgrSet::handle_new_sockets] is not set to anything and can be used to register your custom
/// systems for handling new sockets as highlight above in order to re-order the order of running
pub fn socket_mgr_plugin<C, S>(app: &mut App)
where
    C: SocketMgrConfig<S>,
    S: ConnectionComponent,
{
    app.add_systems(Startup, start_network::<C, S>.in_set(SocketMgrStartNetworkSet::<C, S>::default()))
        .add_systems(
            Update,
            receive_spawned_sockets::<S>
                .run_if(az_startup_succeeded())
                .in_set(SocketMgrSet::<C>::handle_received_socket()),
        )
        .add_systems(PostUpdate, handle_terminate_network::<S>.in_set(SocketMgrSet::<C>::network_termination()));
}

#[derive(Resource)]
pub struct SocketReceiver<S: ConnectionComponent>(pub UnboundedReceiver<NewTcpConnection>, PhantomData<S>);

#[derive(Resource)]
struct TermSender<S: ConnectionComponent>(UnboundedSender<()>, PhantomData<S>);

fn start_network<C, S>(cfg: Res<ConfigMgr<C>>, mut commands: Commands, rt: Res<TokioRuntime>, mut ev_startup_failed: EventWriter<AzStartupFailedEvent>)
where
    C: SocketMgrConfig<S>,
    S: ConnectionComponent,
{
    let (term_snd, term_rcv) = unbounded_channel();
    let (sock_snd, sock_rcv) = unbounded_channel::<NewTcpConnection>();
    commands.insert_resource(SocketReceiver(sock_rcv, PhantomData::<S>));
    commands.insert_resource(TermSender(term_snd, PhantomData::<S>));

    let bind_addr = cfg.retrieve_bind_addr();
    let acceptor = match rt.block_on(TcpListener::bind(bind_addr)) {
        Err(e) => {
            ev_startup_failed.send_default();
            error!(cause=?e, "TCP network startup failed due to acceptor error");
            return;
        },
        Ok(t) => t,
    };

    let sem = cfg.retrieve_max_connections().map(|max| Arc::new(Semaphore::new(max)));
    rt.spawn(accept_sockets::<S>(acceptor, sem, term_rcv, sock_snd));
}

fn handle_terminate_network<S: ConnectionComponent>(mut app_exit_events: EventReader<AppExit>, term_snds: Res<TermSender<S>>) {
    let mut sent_exit = false;
    for _ev in app_exit_events.read() {
        if !sent_exit {
            // NOTE: run asynchronously w/out needing for error handling (For now)
            // this should be short so it seems fairly okay to do
            //
            // This is just an attempt to terminate the accept loop, the program
            // may go ahead and exit anyway via a tokio runtime cancellation.
            debug!("sending network termination signal for connection type {}!", S::connection_type());
            if let Err(e) = term_snds.0.send(()) {
                debug!(cause=?e, "send terminate error, terminate network receiving channel half may be dropped or closed");
            }
            sent_exit = true;
        }
        // We still wanna at least process the rest of the exits anyway, if any.
        // so no break.
    }
}

#[derive(Component)]
pub struct RunStartTcpSocketTask<S>(JoinHandle<CommandQueue>, PhantomData<S>);

impl<S> RunStartTcpSocketTask<S> {
    pub fn new(jh: JoinHandle<CommandQueue>) -> Self {
        Self(jh, PhantomData::<S>)
    }
}

fn receive_spawned_sockets<S: ConnectionComponent>(mut commands: Commands, rt: Res<TokioRuntime>, mut start_tcp_tasks: Query<&mut RunStartTcpSocketTask<S>>) {
    for mut task in &mut start_tcp_tasks {
        if let Some(Ok(mut commands_queue)) = rt.block_on(poll_once(&mut task.0)) {
            // append the returned command queue to have it execute later
            commands.append(&mut commands_queue);
        }
    }
}

async fn accept_sockets<S: ConnectionComponent>(
    acceptor: TcpListener,
    maybe_sem: Option<Arc<Semaphore>>,
    mut term: UnboundedReceiver<()>,
    send_sockets: UnboundedSender<NewTcpConnection>,
) {
    loop {
        // Try to retrieve a semaphore permit first, even if a connection is not retrieved
        let sem_perm = if let Some(sem) = &maybe_sem {
            let p = tokio::select! {
                _ = term.recv() => {
                    debug!("termination triggered, quitting network async accept loop");
                    break;
                },
                res = sem.clone().acquire_owned() => res,
            };
            debug_assert!(
                p.is_ok(),
                "Semaphore for accept sockets is closed, this should never happen, terminating from accept socket loop, {:?}",
                p.err()
            );
            p.ok()
        } else {
            None
        };
        let (conn, addr) = tokio::select! {
            _ = term.recv() => {
                debug!("termination triggered, quitting network async accept loop for connection type {}", S::connection_type());
                break;
            }
            conn = acceptor.accept() => {
                match conn {
                    Err(e) => {
                        error!(cause=?e, "failed to retrieve client connection");
                        continue
                    },
                    Ok(c) => c,
                }
            },
        };

        if let Err(e) = send_sockets.send(NewTcpConnection {
            conn,
            permit: sem_perm,
            name: AddressOrName::Addr(addr),
        }) {
            // Dont treat this as an error for now, mainly b/c it could be that the app is properly quitting
            debug!(cause=?e, "send error, receiving socket receiving channel half may be dropped or closed, quitting network accept loop");
            break;
        }
    }
}
