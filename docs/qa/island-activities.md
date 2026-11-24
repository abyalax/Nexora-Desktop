# Dynamic Island Activities QA

Perilaku spesifik untuk fitur Activities di Dynamic Island.

## Tab/Zone Tersembunyi

### Deskripsi
Activities tab harus dapat disembunyikan atau ditampilkan sebagai opsi di Dynamic Island.

### Test Cases
1. **Toggle visibilitas**
   - Pastikan toggle untuk show/hide Activities berfungsi
   - Tab harus muncul/hilang sesuai setting
   - Transisi harus smooth tanpa jeda

2. **Tab state persistence**
   - State tab tersimpan/di-load dari settings
   - Setelah restart aplikasi, state tab harus sesuai

3. **Tab interaction**
   - Click pada tab harus switch ke Activities view
   - Active tab harus memiliki visual indicator (highlight/border)
   - Tab harus responsive terhadap DPI scaling

## UI Rows untuk 7 Activities Settings

### Settings yang Diuji
1. Music Activity
2. Timer Activity  
3. Download Activity
4. Send Activity
5. Media Activity
6. Shelf Activity
7. Lyrics Activity

### Test Cases per Activity
- Toggle enable/disable per activity
- Icon rendering yang benar
- Text label localization (Indonesia/English)
- Visual feedback saat disabled
- Interaction saat activity aktif (tap untuk expand/collapse)
