# Belajar dari Proyek Rust Web Ini

Dokumen ini menjelaskan proyek web Rust ini dengan bahasa yang ramah pemula. Tujuannya supaya kamu bisa:

- memahami **gambaran besar arsitektur**
- mengerti **peran setiap folder dan file penting**
- melihat **aliran data**: dari HTTP request → handler → service → repository → database → balik lagi ke response

---

## 1. Gambaran Umum Proyek

Proyek ini adalah **REST API sederhana** dengan fitur:

- registrasi user
- login dengan password
- pembuatan **JWT access token**
- endpoint `/profile` yang hanya bisa diakses jika user sudah login (pakai JWT)

Teknologi utama:

- `actix-web` → framework web async
- `sqlx` + PostgreSQL → akses database
- `jsonwebtoken` → membuat dan memverifikasi JWT
- `bcrypt` → hashing password
- `dotenv` → membaca konfigurasi dari file `.env`

Struktur direktori utama:

```text
src
├── main.rs             # titik masuk aplikasi
├── api/                # handler HTTP (controller)
├── config/             # baca konfigurasi (env) ke struct
├── domain/             # model dan tipe domain (User, error, JWT)
├── repositories/       # query ke database
└── services/           # logika bisnis khusus (mis. generate_token)
```

---

## 2. Alur Request Tingkat Tinggi

Mari lihat apa yang terjadi ketika client memanggil `/api/login`:

1. **`main.rs`** menjalankan server Actix dan mendaftarkan route:

   - `/api/register` → `api::auth::register_handler`
   - `/api/login` → `api::auth::login_handler`
   - `/api/profile` → `api::users::profile_handler`

2. Ketika HTTP request masuk ke `/api/login`:

   - Actix memanggil fungsi `login_handler` di `src/api/auth.rs`
   - Handler ini membaca JSON request (username & password)
   - Handler memanggil **repository** untuk mencari user di database
   - Handler memverifikasi password
   - Handler memanggil **service** untuk membuat JWT token
   - Handler mengembalikan JSON response berisi `access_token`

3. Untuk `/api/profile`:

   - Middleware `JwtMiddleware` membaca header `Authorization`
   - Token JWT diverifikasi, dan `user_id` diambil dari `Claims`
   - Handler `profile_handler` memakai `user_id` untuk query ke database
   - Handler mengembalikan data user sebagai JSON

Dengan kata lain:

```text
Request → api::* (handler) → services::* (logika bisnis)
        → repositories::* (database) → balik ke handler → Response
```

---

## 3. `main.rs`: Titik Masuk Aplikasi

File: `src/main.rs`

Peran:

- memulai aplikasi
- membaca konfigurasi (via `AppConfig`)
- membuat koneksi database (pool)
- mendaftarkan route

Potongan penting:

```rust
mod api;
mod domain;
mod services;
mod config;
mod repositories;

use crate::config::AppConfig;
use crate::domain::AppState;
use actix_cors::Cors;
use actix_web::{App, HttpServer, web};
use sqlx::postgres::PgPoolOptions;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = AppConfig::from_env();

    let db_pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .connect(&config.db_url)
        .await
        .expect("Failed to connect to database");

    println!("Server running on http://{}:{}", config.server_host, config.server_port);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .app_data(web::Data::new(AppState {
                db_pool: db_pool.clone(),
                jwt_secret: config.jwt_secret.clone(),
            }))
            .service(
                web::scope("/api")
                    .route("/register", web::post().to(api::auth::register_handler))
                    .route("/login", web::post().to(api::auth::login_handler))
                    .route("/profile", web::get().to(api::users::profile_handler)),
            )
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
```

Konsep penting untuk pemula:

- `mod api;` dsb → mendeklarasikan module (folder/file lain di `src`).
- `AppState` → struct yang berisi state aplikasi (db pool, secret JWT) yang dibagikan ke handler.
- `HttpServer::new(move || { ... })` → closure yang membuat `App` untuk setiap worker.
- `.service(web::scope("/api").route(...))` → mendefinisikan endpoint dan handler.

---

## 4. `config`: Membaca Konfig dari Environment

File: `src/config/mod.rs`

Strukturnya:

```rust
use dotenv::dotenv;
use std::env;

#[derive(Clone)]
pub struct AppConfig {
    pub db_url: String,
    pub jwt_secret: String,
    pub max_connections: u32,
    pub server_host: String,
    pub server_port: u16,
}
```

Metode penting:

```rust
impl AppConfig {
    pub fn from_env() -> Self {
        dotenv().ok();

        let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let max_connection = env::var("MAX_CONNECTIONS").unwrap_or("5".to_string());
        let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
        let server_host = env::var("SERVER_HOST").unwrap_or("127.0.0.1".to_string());
        let server_port = env::var("SERVER_PORT").unwrap_or("8080".to_string());

        Self {
            db_url,
            max_connections: max_connection.parse().unwrap_or(5),
            jwt_secret,
            server_host,
            server_port: server_port.parse().unwrap_or(8080),
        }
    }
}
```

Konsep penting:

- `dotenv().ok();` → membaca variabel dari file `.env`.
- `env::var("NAME")` → mengambil nilai environment variable.
- Struct `AppConfig` menyimpan semua konfigurasi yang dibutuhkan di satu tempat.

---

## 5. `domain`: Model dan Error Aplikasi

File utama: `src/domain/mod.rs`

```rust
use sqlx::PgPool;
pub mod auth;
pub mod error;
pub mod jwt;
pub mod user;

pub use auth::{LoginPayload, LoginResponse, RegisterPayload};
pub use error::AppError;
pub use user::{User, UserResult};

pub struct AppState {
    pub db_pool: PgPool,
    pub jwt_secret: String,
}
```

Di sini:

- `auth.rs` → struct input/output untuk auth (payload login/register, response).
- `user.rs` → struct user yang diambil dari database.
- `error.rs` → definisi error aplikasi.
- `jwt.rs` → definisi JWT claims dan middleware `JwtMiddleware`.
- `AppState` → state global (db pool + JWT secret) yang bisa dipakai di handler.

### 5.1 `domain/user.rs`

```rust
#[derive(Serialize, sqlx::FromRow)]
pub struct User {
    pub id: i32,
    pub username: String,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct UserResult {
    pub id: i32,
    pub username: String,
    pub password_hash: String,
}
```

Konsep:

- `#[derive(Serialize, sqlx::FromRow)]` → otomatis membuat implementasi untuk:
  - `Serialize` → bisa diubah menjadi JSON.
  - `FromRow` → bisa diisi langsung dari hasil query `sqlx`.
- `User` → untuk data yang aman dikirim ke client.
- `UserResult` → menyimpan juga `password_hash` (dipakai internal, tidak dikirim ke client).

### 5.2 `domain/error.rs`

```rust
#[derive(Debug)]
pub enum AppError {
    Internal(String),
    Conflict(String),
    Unauthorized(String),
}
```

`AppError` mengimplementasikan `ResponseError`, sehingga kalau handler mengembalikan `Result<_, AppError>`, Actix otomatis mengubahnya menjadi HTTP response dengan status yang sesuai.

### 5.3 `domain/jwt.rs`

Di sini didefinisikan:

- `Claims` → isi token JWT (user id dan waktu kedaluwarsa).
- `JwtMiddleware` → extractor Actix yang:
  - membaca header `Authorization: Bearer <token>`
  - memverifikasi token dengan `jsonwebtoken`
  - jika valid, menyimpan `Claims` di struct

Handler seperti `profile_handler` cukup menerima parameter `jwt: JwtMiddleware` dan sudah dianggap user terotentikasi.

---

## 6. `api`: Handler HTTP

Folder: `src/api/`

### 6.1 `api/auth.rs`

Berisi dua handler:

- `register_handler` → POST `/api/register`
- `login_handler` → POST `/api/login`

Keduanya:

- menerima JSON (`web::Json<...>`)
- mengakses `AppState` untuk database dan secret JWT
- mapping error dari repository/service ke `AppError`

Contoh bagian penting di `login_handler`:

```rust
let result = find_user_by_username(&state.db_pool, &payload.username)
    .await
    .map_err(|_| AppError::Internal("Failed to login".to_string()))?;

match result {
    Some(user) => {
        let is_password_valid = verify(&payload.password, &user.password_hash)
            .map_err(|_| AppError::Internal("Password verification failed".to_string()))?;
        if is_password_valid {
            let token = auth_service::generate_token(user, &state.jwt_secret)
                .map_err(|_| AppError::Internal("Failed to create token".to_string()))?;
            Ok(HttpResponse::Ok().json(LoginResponse {
                access_token: token,
            }))
        } else {
            Err(AppError::Unauthorized(
                "Invalid username or password".to_string(),
            ))
        }
    }
    None => Err(AppError::Unauthorized(
        "Invalid username or password".to_string(),
    )),
}
```

Di sini terlihat:

- Handler **tidak melakukan query DB langsung**.
- Handler memanggil `repositories::find_user_by_username`.
- Password dicek dengan `bcrypt::verify`.
- JWT dibuat oleh `services::auth_service::generate_token`.

### 6.2 `api/users.rs`

Handler `profile_handler` untuk GET `/api/profile`:

```rust
pub async fn profile_handler(
    jwt: JwtMiddleware,
    state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let user_id = jwt.claims.sub;
    let user = find_user_by_id(&state.db_pool, user_id)
        .await
        .map_err(|_| AppError::Internal("Failed to get user profile".to_string()))?;
    Ok(HttpResponse::Ok().json(user))
}
```

Konsep:

- `jwt: JwtMiddleware` → otomatis diisi jika token valid; kalau tidak valid, handler tidak dijalankan.
- Handler memanggil repository (`find_user_by_id`) untuk mengambil data user.

---

## 7. `repositories`: Akses Database

Folder: `src/repositories/`

Tujuan:

- mengumpulkan semua query `sqlx` di satu tempat
- handler hanya memanggil fungsi-fungsi repository

File `mod.rs`:

```rust
pub mod user_repository;

pub use user_repository::{create_user, find_user_by_id, find_user_by_username};
```

File `user_repository.rs`:

```rust
use crate::domain::{User, UserResult};
use sqlx::PgPool;

pub async fn create_user(
    db_pool: &PgPool,
    username: &str,
    password_hash: &str,
) -> Result<User, sqlx::Error> {
    let created_user = sqlx::query_as!(
        User,
        "INSERT INTO users(username, password_hash) VALUES ($1, $2) RETURNING id, username",
        username,
        password_hash,
    )
    .fetch_one(db_pool)
    .await;
    Ok(created_user?)
}
```

Fungsi lain:

- `find_user_by_username` → pakai `fetch_optional`, mengembalikan `Option<UserResult>`.
- `find_user_by_id` → pakai `fetch_one`, mengembalikan `User`.

Konsep penting:

- Repository **hanya tahu soal database dan `sqlx`**.
- Mapping error DB ke error HTTP (`AppError`) dilakukan di handler.

---

## 8. `services`: Logika Bisnis Tambahan

Folder: `src/services/`

Saat ini ada `auth_service.rs`:

```rust
use crate::AppError;
use crate::domain::jwt::Claims;
use crate::UserResult;
use chrono::{Duration, Utc};
use jsonwebtoken::{EncodingKey, Header, encode};

pub fn generate_token(user: UserResult, jwt_secret: &str) -> Result<String, AppError> {
    let exp = (Utc::now() + Duration::days(1)).timestamp() as usize;
    let claims = Claims { sub: user.id, exp };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_ref()),
    )
    .map_err(|_| AppError::Internal("Failed to create token".to_string()))?;
    Ok(token)
}
```

Konsep:

- Service ini bertugas membuat JWT token dari `UserResult`.
- Dengan memisahkan ke service, handler login jadi lebih bersih.

---

## 9. Ide Latihan untuk Memperdalam

Beberapa latihan yang bisa kamu coba:

1. **Tambah field baru di user**

   - Tambah kolom `email` di tabel dan di `User` / `UserResult`.
   - Sesuaikan query di repository.
   - Kembalikan `email` di `/profile`.

2. **Tambah endpoint baru**

   Misalnya `GET /api/me` yang melakukan hal yang sama dengan `/profile`, tapi mungkin mengembalikan format berbeda.

3. **Pisahkan AppState**

   Sekarang `AppState` ada di `domain`. Coba pindahkan ke module `infrastructure` atau module `app` sendiri agar domain benar-benar bebas dari framework.

4. **Tambah service baru**

   Misalnya `user_service` dengan fungsi:

   - `get_profile(user_id)` → memanggil repository, mapping error, dan mengembalikan `User`.
   - Lalu `profile_handler` memanggil `user_service::get_profile`.

Dengan memahami struktur dan alur proyek ini, kamu sudah punya fondasi yang sangat bagus untuk proyek web Rust yang lebih besar dan kompleks.

