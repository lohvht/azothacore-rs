#[macro_export]
macro_rules! row_vector_to_matrix_index {
    ( S: $matrix_shape:expr, $i:expr ) => {{
        let (nrows, ncols) = $matrix_shape;
        // Two most common uses that should be optimized by the compiler for statically-sized
        // matrices.
        if nrows == 1 {
            ($i, 0)
        } else if ncols == 1 {
            (0, $i)
        } else {
            ($i / ncols, $i % ncols)
        }
    }};
    ( $matrix:expr, $i:expr ) => {{
        $crate::row_vector_to_matrix_index!(S: $matrix.shape(), $i)
    }};
}
