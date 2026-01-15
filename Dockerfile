# --- Stage 1: Builder ---
# Menggunakan image Rust resmi sebagai dasar untuk build
FROM rust:1.78-slim AS builder

# Membuat direktori kerja di dalam container
WORKDIR /usr/src/app

# Menyalin file manifes proyek
COPY Cargo.toml Cargo.lock ./

# Membuat "dummy" src/main.rs untuk men-cache dependensi
# Ini adalah trik untuk memanfaatkan layer caching Docker.
# Dependensi hanya akan di-build ulang jika Cargo.toml berubah.
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release

# Menyalin seluruh kode sumber proyek
COPY src ./src

# Membersihkan dummy file dan membangun aplikasi secara penuh
RUN rm -f target/release/deps/rust_lesson*
RUN cargo build --release

# --- Stage 2: Final Image ---
# Menggunakan image dasar yang kecil untuk image akhir
FROM debian:bullseye-slim

# Menetapkan argumen non-interaktif untuk instalasi paket
ARG DEBIAN_FRONTEND=noninteractive

# Menginstal sertifikat CA yang diperlukan untuk koneksi HTTPS/TLS
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

# Menyalin binary yang sudah dikompilasi dari stage builder
COPY --from=builder /usr/src/app/target/release/rust-lesson /usr/local/bin/rust-lesson

# Menetapkan user non-root untuk keamanan
RUN useradd -ms /bin/bash appuser
USER appuser

# Menetapkan direktori kerja untuk user baru
WORKDIR /home/appuser

# Mengekspos port yang digunakan oleh aplikasi
EXPOSE 8080

# Perintah untuk menjalankan aplikasi saat container dimulai
CMD ["rust-lesson"]
