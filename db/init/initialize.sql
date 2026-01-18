-- Skrip ini akan dijalankan secara otomatis saat container database pertama kali dibuat.

-- Membuat tabel 'users' jika belum ada
CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL
);

-- Anda bisa menambahkan data awal di sini jika perlu
-- INSERT INTO users (username, password_hash) VALUES ('admin', 'hashed_password_for_admin');
