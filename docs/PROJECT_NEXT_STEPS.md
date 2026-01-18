# Rekomendasi Pengembangan Selanjutnya

Dokumen ini berisi daftar pengembangan yang direkomendasikan supaya proyek ini semakin terstruktur, clean, dan mendekati standar yang biasa dipakai di perusahaan.

Urutan disusun dari yang lebih dasar/mudah ke yang lebih advanced.

---

## 1. Rapikan Batas Layer (Domain vs Framework)

- **Pindahkan `AppState` keluar dari `domain`**
  - Saat ini `AppState` didefinisikan di `domain::mod` dan berisi:
    - `PgPool` (detail infrastruktur database)
    - `jwt_secret` (konfigurasi/security)
  - Rekomendasi:
    - Buat module baru, misalnya:
      - `src/app_state.rs` atau
      - `src/infrastructure/state.rs`
    - Pindahkan `AppState` ke module tersebut.
  - Tujuan:
    - `domain` fokus pada model dan aturan bisnis murni.
    - Detail framework dan infrastruktur dipisahkan ke layer lain.

- **Bersihkan `domain` dari ketergantungan ke framework**
  - Pastikan module di `domain`:
    - tidak meng-import `actix_web`
    - tidak meng-import `jsonwebtoken` (kecuali untuk tipe data murni, seperti struct `Claims` tanpa logika HTTP)
  - Detail seperti middleware Actix (`FromRequest`) sebaiknya berada di:
    - `api::middleware`
    - atau `infrastructure::http`

---

## 2. Perjelas Pemisahan API → Service → Repository

- **Tambahkan `user_service` di layer service**
  - Buat file `src/services/user_service.rs` yang berisi fungsi seperti:
    - `get_profile(user_id, &PgPool) -> Result<User, AppError>`
    - `register_user(payload, &PgPool) -> Result<User, AppError>`
  - Handler di `api::users` dan `api::auth`:
    - tidak lagi berisi logika bisnis yang banyak
    - cukup memanggil service dan mengubah hasil menjadi `HttpResponse`

- **Definisikan trait `UserRepository` di domain (opsional tapi bagus)**
  - Di `domain`, definisikan:
    - `trait UserRepository { async fn create_user(...); async fn find_by_username(...); async fn find_by_id(...); }`
  - Di `repositories`, buat implementasi konkret:
    - `struct PostgresUserRepository { pool: PgPool }`
    - `impl UserRepository for PostgresUserRepository { ... }`
  - Service menerima parameter generik:
    - `impl UserRepository` alih-alih langsung `PgPool`
  - Keuntungan:
    - mudah dibuat fake/in-memory repo saat unit test
    - domain tidak tergantung pada SQLx atau PostgreSQL

---

## 3. Konsistensi Penamaan dan Module Path

- **Selaraskan penamaan field konfigurasi**
  - Di `AppConfig`:
    - gunakan nama field yang jelas dan konsisten, misalnya:
      - `database_url` alih-alih `db_url`
  - Hal ini memudahkan pemetaan:
    - env `DATABASE_URL` → field `database_url`

- **Konsisten dalam meng-import repository**
  - Di `repositories/mod.rs` sudah ada:
    - `pub use user_repository::{create_user, find_user_by_id, find_user_by_username};`
  - Rekomendasi:
    - di handler, gunakan pola yang sama:
      - `use crate::repositories::{create_user, find_user_by_id, find_user_by_username};`
  - Tujuan:
    - cara import seragam di seluruh project
    - mudah ditebak dan dicari

---

## 4. Centralize Error Handling Lebih Jauh

- **Perluas `AppError` jika aplikasi berkembang**
  - Tambah varian baru bila perlu:
    - `Validation(String)`
    - `NotFound(String)`
    - `Forbidden(String)`
  - Sesuaikan implementasi `ResponseError` untuk mengembalikan HTTP status yang tepat.

- **Pertimbangkan `From<sqlx::Error> for AppError`**
  - Implementasi konversi:
    - `impl From<sqlx::Error> for AppError { ... }`
  - Keuntungan:
    - di service dan repository bisa menggunakan operator `?` langsung
    - mengurangi boilerplate `.map_err(...)` yang berulang

---

## 5. Gunakan `startup.rs` untuk Wiring Server

- **Pendekkan `main.rs`**
  - Biarkan `main.rs` fokus kepada:
    - membaca konfigurasi (`AppConfig::from_env`)
    - membuat koneksi database (`PgPool`)
    - memanggil fungsi `startup::run(config, db_pool)`

- **Pindahkan konfigurasi `HttpServer` ke `startup.rs`**
  - Di `startup.rs`, buat fungsi:
    - `pub fn run(config: AppConfig, db_pool: PgPool) -> std::io::Result<Server> { ... }`
  - Fungsi ini mengatur:
    - CORS
    - `AppState`
    - registrasi semua route
  - Keuntungan:
    - mudah dibuat integration test:
      - test bisa memanggil `run(...)` dengan konfigurasi test
    - struktur mirip dengan pola di banyak perusahaan

---

## 6. Dokumentasi dan Contoh Request

- **Lengkapi dokumentasi endpoint**
  - Di salah satu file markdown (misalnya `PROJECT_LEARNING_NOTES.md` atau file baru):
    - tulis daftar endpoint:
      - `POST /api/register`
      - `POST /api/login`
      - `GET /api/profile`
    - sertakan:
      - contoh request dengan `curl`
      - contoh body JSON
      - contoh response JSON

- **Tambahkan diagram alur singkat (teks)**
  - Misalnya:
    - alur login dari client → handler → repository → service → JWT → client
    - alur profile dari JWT → middleware → repository → handler → client

Tujuannya:
- memudahkan pemula memahami alur tanpa harus membaca semua kode
- membantu diri sendiri beberapa bulan ke depan saat kembali ke project ini

---

## 7. Testing Dasar

- **Integration test minimal**
  - Buat folder `tests/` dan tambahkan test:
    - spin up server dengan konfigurasi test
    - panggil `/api/register` dan `/api/login`
    - pastikan login mengembalikan `access_token`

- **Unit test untuk komponen murni**
  - Test `AppConfig::from_env` dengan environment dummy
  - Test service seperti `generate_token`:
    - pastikan `Claims.sub` dan `Claims.exp` berisi nilai yang benar setelah decode

Dengan latihan test sederhana:
- kamu belajar cara menulis test di Rust
- project terasa lebih “production-ready”

---

## 8. Hardening JWT dan Keamanan

- **Perkaya `Claims`**
  - Tambahkan field seperti:
    - `iss` (issuer)
    - `aud` (audience)
    - `iat` (issued at)
  - Sesuaikan konfigurasi `Validation` di JWT middleware.

- **Pisahkan konfigurasi security**
  - Buat struct seperti `SecurityConfig` di `config`:
    - menyimpan `jwt_secret`, mungkin juga `jwt_issuer`, `jwt_audience`
  - `AppState` menyimpan `SecurityConfig`
  - Middleware dan service auth membaca pengaturan dari struct ini

---

## 9. Formatter dan Linting

- **Gunakan `cargo fmt`**
  - Menjaga format kode konsisten secara otomatis.
- **Gunakan `cargo clippy`**
  - Memberi saran perbaikan idiomatik Rust
  - Di banyak perusahaan:
    - `cargo fmt` dan `cargo clippy -- -D warnings` dijalankan di CI

Dengan demikian:
- gaya kode konsisten
- banyak kesalahan kecil/anti-pattern bisa dideteksi sejak awal

