use matrix::Matrix;

use crate::matrix::NonZeroSquareMatrix;


mod matrix;
mod numeric;

pub type Type = f32;


fn main() {
    let matrix: Vec<Vec<Type>> = serde_json::from_slice(&std::fs::read("in.json").unwrap()).unwrap();
    let matrix = Matrix::try_from(matrix).unwrap();
    let matrix = NonZeroSquareMatrix::try_from(matrix).unwrap();
    let det = matrix.det();
    let reversed = matrix.algebraic_additions().unwrap().transposed() / det; 
    println!("{:?}", reversed);
    // let matrix = matrix.transpose();
    // println!("{:?}", matrix);
    // let matrix: NonZeroSquareMatrix<_> = matrix.try_into().unwrap();
    // println!("{:?}", matrix.algebraic_additions());
    // let matrix: Matrix<_> = matrix.exclude(1, 1).unwrap().into();
    // let data = serde_json::to_string::<Vec<Vec<Type>>>(&matrix.into()).unwrap();
    // std::fs::write("out.json", data);
}

