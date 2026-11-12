# Auto Download from GitHub

## English

A small command-line program that downloads a file from a URL and saves it in
the current working directory.

### Requirements

- Rust toolchain with Cargo

### Run

```bash
cargo run
```

Enter a URL when prompted. The program saves the response using the filename
provided by the server (`Content-Disposition`) when available; otherwise, it
uses the filename from the URL. If neither provides a usable name, it uses
`download`.

The program preserves a file extension from the name or URL. HTML responses
are saved with the `.html` extension. If a file has no extension, PDF and ZIP
content types are used to choose one; other unknown types use `.bin`.
Downloaded files are written to the directory from which the program is run.

### Tests

```bash
cargo test
```

## Русский

Небольшая консольная программа для загрузки файла по URL и сохранения его в
текущую рабочую папку.

### Требования

- Rust и Cargo

### Запуск

```bash
cargo run
```

Введите URL после появления запроса. Программа сохраняет ответ под именем,
указанным сервером в `Content-Disposition`, если оно есть; иначе используется
имя из URL. Если подходящее имя отсутствует, файл будет назван `download`.

Расширение берётся из имени файла или URL. HTML-страницы сохраняются с
расширением `.html`. Если расширения нет, для PDF и ZIP оно определяется по
типу содержимого; для остальных неизвестных типов используется `.bin`.
Загруженный файл сохраняется в папку, из которой запущена программа.

### Тесты

```bash
cargo test
```
