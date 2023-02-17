use clap::Parser; // для работы clap требуется его Parser в пространстве имен
use miette::{Diagnostic, IntoDiagnostic, NamedSource, SourceOffset};
use std::{borrow::Cow, path::PathBuf}; // структура пути в файловой системе
use thiserror::Error;

// "clap" воспринимает документацию как справочную информацию и запишет в консольном интерфейсе
/// Приложение для обращения матриц
#[derive(Parser)]
struct Cli {
    /// файл, из которого будет получена матрица
    #[clap(default_value = "in.json")]
    input: PathBuf,

    /// файл, в который будет записан результат
    #[clap(default_value = "out.json")]
    output: PathBuf,
}

#[derive(Debug, Diagnostic, Error)]
#[error("Ошибка чтения матрицы")]
struct MatrixParseError {
    #[source_code]
    file: NamedSource,

    #[help]
    message: String,

    #[label]
    point: SourceOffset,
}

fn main() -> miette::Result<()> {
    let Cli { input, output } = Cli::parse();

    // чтение файла и оборачивание его в Clone-on-write контейнер
    let file = Cow::Owned(std::fs::read_to_string(&input).into_diagnostic()?);

    let source = NamedSource::new(input.as_os_str().to_string_lossy(), Cow::clone(&file));
    // парсинг данных
    let data = serde_json::from_str::<Vec<Vec<f64>>>(&file).map_err(|err| {
        // в случае ошибки, она трансформируется в информативный отчет
        MatrixParseError {
            file: source,
            message: format!("{err}"),
            point: SourceOffset::from_location(Cow::clone(&file), err.line(), err.column()),
        }
    })?;

    // получение размеров матрицы
    let nrows = data.len(); // строки
    let ncols = data
        .get(0) // так как строк может не быть, get возвращает Option<Vec<_>>
        .map(Vec::len) // map применяет Vec::len только в случаае наличия значения
        .unwrap_or(0); // если значения нет, вместо значения будет поставлен 0

    // проверка матрицы на вырожденность
    if (nrows == 1 || nrows == 0) && ncols == 0 {
        return Err(miette::Report::msg("нельзя обратить вырожденную матрицу"));
    }

    // проверка матрицы на неквадратность
    if nrows != ncols {
        return Err(miette::Report::msg("нельзя обратить не квадратную матрицу"));
    }

    // проверка того, что все строки матрицы равной длины
    if data.iter().map(Vec::len).any(|len| ncols != len) {
        return Err(miette::Report::msg(
            "не все строки матрицы имеют одинаковую длину",
        ));
    }

    // получение матрицы nalgebra
    let matrix = nalgebra::DMatrix::from_iterator(nrows, ncols, data.into_iter().flatten());
    // инверсия матрицы
    let matrix = matrix
        .try_inverse()
        .ok_or_else(|| miette::Report::msg("инверсия матрицы не удалась"))?;

    let mut data = Vec::with_capacity(nrows);
    for row in matrix.as_slice().chunks(ncols) {
        data.push(Vec::from(row))
    }

    // запись в файл
    std::fs::write(output, serde_json::to_vec(&data).into_diagnostic()?).into_diagnostic()
}
