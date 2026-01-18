# Struktur Proyek Web Development dengan Rust

Dokumen ini mendeskripsikan contoh struktur proyek web Rust yang umum dipakai di perusahaan: **scalable**, **rapi**, dan **mudah dipelajari**. Contoh ini diasumsikan menggunakan stack seperti:

- `actix-web` (web framework)
- `sqlx` + PostgreSQL (akses data)
- `serde` untuk serialisasi
- `dotenv` / environment variable untuk konfigurasi

Direktori yang belum ada di proyek saat ini bisa ditambahkan bertahap ketika fitur tersebut diperlukan.

---

## Gambaran Umum Struktur

```text
.
├── Cargo.toml
├── Cargo.lock
├── src
│   ├── main.rs
│   ├── startup.rs
│   ├── config
│   │   └── mod.rs
│   ├── api
│   │   ├── mod.rs
│   │   ├── auth.rs
│   │   └── users.rs
│   ├── domain
│   │   ├── mod.rs
│   │   └── user.rs
│   ├── services
│   │   ├── mod.rs
│   │   ├── auth_service.rs
│   │   └── user_service.rs
│   ├── infrastructure
│   │   ├── mod.rs
│   │   ├── db.rs
│   │   └── repositories
│   │       ├── mod.rs
│   │       └── user_repository.rs
│   └── telemetry.rs
├── migrations/
├── Dockerfile
├── docker-compose.yml
├── .env.example
└── PROJECT_STRUCTURE.md
```

---

## Penjelasan Direktori dan File

### Root

- **Cargo.toml**  
  Definisi paket, dependency, dan fitur. Satu sumber kebenaran untuk konfigurasi build.

- **Cargo.lock**  
  Mengunci versi dependency yang dipakai di environment produksi agar build konsisten.

- **Dockerfile**  
  Definisi image untuk menjalankan aplikasi (multi-stage build: builder + runtime).

- **docker-compose.yml**  
  Orkestrasi service (aplikasi + database + dependency lain).

- **.env.example**  
  Contoh variabel environment yang dibutuhkan (DATABASE_URL, JWT_SECRET, dsb).

- **PROJECT_STRUCTURE.md**  
  Dokumentasi struktur proyek (file ini), memudahkan onboarding developer baru.

### `src/main.rs`

Titik masuk aplikasi. Biasanya hanya berisi:

- bootstrap awal (membaca konfigurasi dasar)
- pemanggilan fungsi `startup::run(...)`
- penanganan error tingkat atas

Tujuannya: `main.rs` tetap pendek dan mudah dibaca.

### `src/startup.rs`

Berisi fungsi untuk:

- membangun instance `HttpServer`/`App` (routing, middleware, CORS)
- menghubungkan `AppState` (pool database, secret key, konfigurasi)

Dengan memindahkan konfigurasi server ke file terpisah, kita bisa:

- mengetes wiring aplikasi tanpa menjalankan `main`
- menambah endpoint baru tanpa mengotori `main.rs`

### `src/config`

Tujuan: satu tempat untuk seluruh konfigurasi aplikasi.

Contoh isi:

- membaca `env` (DATABASE_URL, JWT_SECRET, MAX_CONNECTIONS, dsb)
- mapping ke struct `Settings`
- fungsi helper untuk memuat dan memvalidasi konfigurasi

Keuntungan:

- memudahkan perbedaan konfigurasi antara `local`, `staging`, dan `production`
- menghindari `env::var(...)` tersebar di seluruh kode

### `src/api`

Lapisan HTTP / REST:

- **`mod.rs`**: mengumpulkan dan mendaftarkan semua route (auth, users, dsb)
- **`auth.rs`**: handler endpoint terkait autentikasi (login, register, refresh token)
- **`users.rs`**: handler endpoint untuk resource user

Karakteristik:

- hanya berurusan dengan `HttpRequest`, `HttpResponse`, `Json<T>`, status code
- delegasikan logika bisnis ke lapisan `services`

### `src/domain`

Lapisan domain model (business core):

- struct seperti `User`, `UserId`, dan value object lain
- aturan bisnis yang murni (tanpa bergantung ke framework atau database)

Keuntungan:

- domain mudah dites (unit test) tanpa perlu database
- perubahan di lapisan infrastruktur tidak memaksa mengubah domain

### `src/services`

Lapisan use-case / application service:

- **`auth_service.rs`**: proses login, pembuatan token, validasi credential
- **`user_service.rs`**: proses create/read/update user, dsb

Ciri:

- menerima dependency abstraksi repository (trait dari `domain` atau `infrastructure`)
- menggabungkan beberapa repository + komponen lain untuk menyelesaikan satu use-case

### `src/infrastructure`

Lapisan integrasi dengan dunia luar:

- **`db.rs`**  
  Inisialisasi `PgPool`, konfigurasi koneksi, retry policy.

- **`repositories`**  
  Implementasi konkrit repository (misalnya `UserRepositoryPostgres`) yang memakai `sqlx` untuk query.

Prinsip:

- interface/trait boleh didefinisikan di domain atau di sini
- implementasi konkrit (Postgres, Redis, dsb) hanya ada di lapisan infrastruktur

### `src/telemetry.rs`

Opsional tetapi umum di perusahaan:

- setup logging (env_logger, tracing)
- setup metrics (Prometheus, OpenTelemetry)

Dengan memisahkan telemetry, kita bisa:

- mengontrol format dan level log secara konsisten
- menambah observability tanpa menyentuh logic bisnis

### `migrations/`

Direktori untuk:

- file migrasi database (`.sqlx`, `.sql`, atau format lain tergantung tool)
- sinkron dengan query di code (terutama jika memakai `sqlx` dengan `cargo sqlx prepare`)

Biasanya dikelola melalui:

- `sqlx-cli`
- atau tool migrasi lain yang dipakai tim

---

## Alur Tinggi Permintaan HTTP

1. Request masuk ke server (`main.rs` → `startup.rs`).
2. Route di `src/api` memanggil handler yang sesuai.
3. Handler meneruskan ke service di `src/services`.
4. Service menggunakan domain model (`src/domain`) dan repository (`src/infrastructure`).
5. Data diambil/ditulis ke database melalui `sqlx`.
6. Service mengembalikan hasil ke handler.
7. Handler mengubah hasil menjadi HTTP response (JSON, status code, dsb).

Dengan pemisahan lapisan seperti ini, proyek:

- mudah diskalakan (tinggal menambah modul `api`, `services`, `infrastructure` baru)
- rapi (tanggung jawab tiap lapisan jelas)
- mudah dipelajari (struktur folder mencerminkan arsitektur logis aplikasi)

