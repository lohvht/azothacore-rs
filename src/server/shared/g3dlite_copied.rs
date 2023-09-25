//! copied G3Dlite functions that doesn't exist yet on nalgebra (or exist but
//! in other forms because I'm bad at maths)
//!
//! Generally contains rotation functions
//!
use std::{fmt, ops};

use nalgebra::Matrix3;
use num::{Float, NumCast};

pub fn matrix3_from_euler_angles_xyz<F>(f_y_angle: F, f_p_angle: F, f_r_angle: F) -> Matrix3<F>
where
    F: Float + fmt::Debug + ops::AddAssign + ops::MulAssign + 'static,
{
    let f_cos = f_y_angle.cos();
    let f_sin = f_y_angle.sin();
    #[rustfmt::skip]
    let k_x_mat = Matrix3::new(
        NumCast::from(1.0).unwrap(), NumCast::from(0.0).unwrap(),   NumCast::from(0.0).unwrap(),
        NumCast::from(0.0).unwrap(), f_cos,                         -f_sin,
        NumCast::from(0.0).unwrap(), f_sin,                         f_cos,
    );

    let f_cos = f_p_angle.cos();
    let f_sin = f_p_angle.sin();
    #[rustfmt::skip]
    let k_y_mat = Matrix3::new(
        f_cos,                          NumCast::from(0.0).unwrap(), f_sin,
        NumCast::from(0.0).unwrap(),    NumCast::from(1.0).unwrap(), NumCast::from(0.0).unwrap(),
        -f_sin,                         NumCast::from(0.0).unwrap(), f_cos,
    );

    let f_cos = f_r_angle.cos();
    let f_sin = f_r_angle.sin();
    #[rustfmt::skip]
    let k_z_mat = Matrix3::new(
        f_cos,                          -f_sin,                         NumCast::from(0.0).unwrap(),
        f_sin,                          f_cos,                          NumCast::from(0.0).unwrap(),
        NumCast::from(0.0).unwrap(),    NumCast::from(0.0).unwrap(),    NumCast::from(1.0).unwrap(),
    );

    k_x_mat * (k_y_mat * k_z_mat)
}

pub fn matrix3_from_euler_angles_zyx<F>(f_y_angle: F, f_p_angle: F, f_r_angle: F) -> Matrix3<F>
where
    F: Float + fmt::Debug + ops::AddAssign + ops::MulAssign + 'static,
{
    let f_cos = f_y_angle.cos();
    let f_sin = f_y_angle.sin();
    #[rustfmt::skip]
    let k_z_mat = Matrix3::new(
        f_cos,                          -f_sin,                         NumCast::from(0.0).unwrap(),
        f_sin,                          f_cos,                          NumCast::from(0.0).unwrap(),
        NumCast::from(0.0).unwrap(),    NumCast::from(0.0).unwrap(),    NumCast::from(1.0).unwrap(),
    );

    let f_cos = f_p_angle.cos();
    let f_sin = f_p_angle.sin();
    #[rustfmt::skip]
    let k_y_mat = Matrix3::new(
        f_cos,                          NumCast::from(0.0).unwrap(), f_sin,
        NumCast::from(0.0).unwrap(),    NumCast::from(1.0).unwrap(), NumCast::from(0.0).unwrap(),
        -f_sin,                         NumCast::from(0.0).unwrap(), f_cos,
    );

    let f_cos = f_r_angle.cos();
    let f_sin = f_r_angle.sin();
    #[rustfmt::skip]
    let k_x_mat = Matrix3::new(
        NumCast::from(1.0).unwrap(), NumCast::from(0.0).unwrap(),   NumCast::from(0.0).unwrap(),
        NumCast::from(0.0).unwrap(), f_cos,                         -f_sin,
        NumCast::from(0.0).unwrap(), f_sin,                         f_cos,
    );

    k_z_mat * (k_y_mat * k_x_mat)
}
