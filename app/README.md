# app — Gestión de Boxes (cliente Tauri 2)

App de escritorio instalada en todos los PCs (incluido el servidor).

- Diseño y comportamiento: [`docs/05-app.md`](../docs/05-app.md)
- Tareas: [`tasks/fase-3`](../tasks/fase-3)

```bash
npm install
npm run tauri dev      # requiere la API corriendo (ver README raíz)
npm run check          # tipos TypeScript
npm run tauri build    # instalador .msi (en Windows)
```

- `src/` — interfaz en TypeScript sin framework (solo `invoke`, nunca red directa).
- `src-tauri/` — backend Rust: cliente HTTP, descubrimiento UDP, `cliente.json`, exportar CSV.
- La configuración local queda en la carpeta de configuración de la app
  (`%APPDATA%\cl.cesfam.gestionbox\cliente.json`; en Linux `~/.config/cl.cesfam.gestionbox/`).
