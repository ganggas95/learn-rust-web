# Rules AI-Agent – Rust + Actix (≤1000 chars)

- Gunakan **actix-web** (bukan framework lain) untuk HTTP server.
- Ikuti arsitektur:
  - `api` → handler HTTP (tipis, tanpa SQL)
  - `services` → logika bisnis
  - `repositories` → query `sqlx`
  - `domain` → model, DTO, error
  - `config` → baca ENV ke `AppConfig`
- Query database **wajib** di `repositories/*`, return `Result<T, sqlx::Error>` atau `Result<Option<T>, sqlx::Error>`.
- Handler mengembalikan `Result<HttpResponse, AppError>` dan hanya memanggil service/repository.
- `AppError` adalah error utama; mapping ke status HTTP lewat `ResponseError`.
- JWT:
  - generate token di service (mis. `auth_service`)
  - verifikasi token di extractor/middleware (mis. `JwtMiddleware`)
- Konfigurasi: tidak boleh `env::var` di handler; pakai `AppConfig::from_env()`.
- Setiap perubahan signifikan: jalankan `cargo fmt`, `cargo check` (opsional `cargo clippy`).