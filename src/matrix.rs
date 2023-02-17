use std::{ops::{Div}, num::NonZeroUsize};

use crate::numeric::Numeric;

#[derive(Debug, Clone)]
pub struct Matrix<T> {
    pub array: Vec<T>,
    rows: usize,
    cols: usize,
}


// объявления методов, для всех матриц, элементы которых можно клонировать
impl<T: Clone> Matrix<T> {
    fn get(&self, row: usize, col: usize) -> Option<T> {
        self.array
            .get(row * self.rows + col) // получает элемент массива (Option<&T>)
            .map(Clone::clone) // клонирует элемент (Option<&T> => Option<T>)
    }
}



impl<T> TryFrom<Vec<Vec<T>>> for Matrix<T> {
    type Error = String;

    #[inline] // подсказка компилятору сохранить код этой функции чтобы облегчить оптимизации типа inline
    fn try_from(value: Vec<Vec<T>>) -> Result<Self, Self::Error> {
        let rows = value.len(); // основной массив хранит массивы строки, поэтому его длина - количество строк
        let cols = value
            .get(0) // получение первой строки
            .map(Vec::len) // отображение строки в длину (Option<&Vec<T>> => Option<usize>)
            .unwrap_or(0); // так как строк может не быть, нужно установить fallback значение (Option<usize> => usize)

        // так как у введенная матрица может быть некорректна, этот вариант нужно отбросить
        if value
            .iter() // получение итератора по элементам массива (Vec<Vec<T>> => Iterator<&Vec<T>>)
            .map(Vec::len) // отображение строки в длину (&Vec<T> => usize)
            .any(|len| len != cols) // проверка на равенство длин всех строк 
            {
            return Err(format!("Не все строки в матрице имеют равную длину {cols}"));
        }

        let array = value
            .into_iter() // захват владения над массивом и преобразование в итератор (Vec<Vec<T>> => Iterator<Vec<T>>)
            .flatten() // соединяет все массивы чтобы эдементы шли друг за другом (Iterator<Vec<T>> => Iterator<T>)
            .collect(); // собирает массив из итератора (Iterator<T> => Vec<T>)

        Ok(Matrix { array, rows, cols })
    }
}

impl<T: Clone> From<Matrix<T>> for Vec<Vec<T>> {
    fn from(mat: Matrix<T>) -> Self {
        let mut out = Vec::with_capacity(mat.rows); // with_capacity позволяет выделить достаточно места за одно обращение

        for row_no in 0..mat.rows { // цикл по количеству строк
            let mut row = Vec::with_capacity(mat.cols);

            for col_no in 0..mat.cols { // цикл по количеству столбцов
                row.push(mat.get(row_no, col_no).unwrap())
            }

            out.push(row);
        }
        out
    }
}

#[derive(Debug, Clone)]
pub struct NonZeroSquareMatrix<T> {
    array: Vec<T>,
    size: NonZeroUsize
}

impl<T> TryFrom<Matrix<T>> for NonZeroSquareMatrix<T> {
    type Error = &'static str;

    fn try_from(matrix: Matrix<T>) -> Result<Self, Self::Error> {
        let Matrix { array, rows, cols } = matrix;
        if rows == cols {
            if let Some(size) = NonZeroUsize::new(rows) {
                Ok(Self{ array, size })
                } else {
                    Err("данная матрица является вырожденной")
                }
        } else {
            Err("данная матрица не является квадратной")
        }
    }
}


impl<T> From<NonZeroSquareMatrix<T>> for Matrix<T> {
    fn from(matrix: NonZeroSquareMatrix<T>) -> Self {
        let NonZeroSquareMatrix { array, size } = matrix;
        Self { array, rows: size.get(), cols: size.get() }
    }
}


impl<T: Numeric> Div<T> for NonZeroSquareMatrix<T> {
    type Output = NonZeroSquareMatrix<T>;

    fn div(mut self, rhs: T) -> Self::Output {
        for elem in &mut self.array {
            *elem /= rhs;
        }
        self
    }
}

// для любой матрицы, элементы которой:
impl<T: Numeric> NonZeroSquareMatrix<T>{
    pub fn det(&self) -> T {
        let size = self.size.get();
        match size {
            0 => unreachable!(),
            1 => self.array[0],
            2 => self.array[0] * self.array[3] - self.array[1] * self.array[2],
            size => {
                let mut res = T::ZERO;
                let row_no = 0;
                for col_no in 0..size {
                    let sign = if col_no % 2 == 0 { T::ONE } else { T::ZERO - T::ONE };
                    res += self.array[index(size, row_no, col_no)] * self.exclude(row_no, col_no).unwrap().det() * sign;
                };
                res
            },
        }
    }

    pub fn exclude(&self, row_no: usize, col_no: usize) -> Option<NonZeroSquareMatrix<T>> {
        let size = self.size.get();
        let new_size = NonZeroUsize::new(size - 1)?;
        let mut array = Vec::with_capacity((size - 1).pow(2));
        for row_no_old in 0..size {
            for col_no_old in 0..size {
                if (row_no != row_no_old) && (col_no != col_no_old) {
                    array.push(self.array[index(size, row_no_old, col_no_old)])
                }
            }
        }
        Some(Self { array, size: new_size })
    }

    pub fn algebraic_additions(self) -> Option<Self> {
        let size = self.size.get();
        if size == 1 { return None; }
        let mut array = Vec::with_capacity(size * size);
        for row_no in 0..size {
            for col_no in 0..size {
                let sign = if (row_no + col_no) % 2 == 1 { T::ZERO - T::ONE } else { T::ONE };
                array.push(self.exclude(row_no, col_no).unwrap().det() * sign)
            }
        }
        Some(Self { array, size: self.size })
    } 

    /// транспонирование матрицы
    pub fn transposed(mut self) -> Self {
        let size = self.size.get();
        for row_no in 0..size {
            for col_no in row_no + 1..size {
                let left = index(size, row_no, col_no);
                let right = index(size, col_no, row_no);
                self.array.swap(left, right);
            }
        }
        self
    }
}



/// отображает индексы строки и столбца в индекс внутреннего массива
///```
/// assert_eq!(index(3, 1, 1), 4);
/// assert_eq!(index(3, 2, 2), 8);
/// assert_eq!(index(3, 10, 10), 40);
/// ```
// эта функция не объявлена как `pub`, поэтому из других модулей доступна не будет
fn index(row_count: usize, row_no: usize, col_no: usize) -> usize { row_no * row_count + col_no }


#[cfg(test)]
mod tests {
    use super::{Matrix, NonZeroSquareMatrix};

    #[test]
    fn vec_convert() {
        let vec = vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]];
        let matrix: Matrix<_> = vec.clone().try_into().unwrap();
        let later: Vec<Vec<_>> = matrix.into();

        assert_eq!(vec, later) 
    }

    #[test]
    #[should_panic]
    fn vec_convert_fail() {
        let vec = vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9, 0]];
        let _: Matrix<_> = vec.try_into().unwrap();
    }

    #[test]
    #[should_panic]
    fn to_zero() {
        let matrix: Matrix<i32> = vec![].try_into().unwrap();
        NonZeroSquareMatrix::try_from(matrix).unwrap();
    }

    #[test]
    #[should_panic]
    fn to_non_square() {
        let matrix: Matrix<_> = vec![vec![1, 2, 3]].try_into().unwrap();
        NonZeroSquareMatrix::try_from(matrix).unwrap();
    }

    #[test]
    fn to_non_zero_square() {
        let vec = vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]];
        let matrix: Matrix<_> = vec.clone().try_into().unwrap();
        let non_zero_square = NonZeroSquareMatrix::try_from(matrix).unwrap();
        let later = Matrix::from(non_zero_square);
        let later_vec: Vec<Vec<_>> = later.into();

        assert_eq!(vec, later_vec)
    }

}

