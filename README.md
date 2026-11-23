# AlemanIusLexAI

Plataforma de gestión de documentos para Despachos legales y Buffetes Juridicos[cite: 3], respaldada en **Solana** mediante un programa inteligente desarrollado con **Anchor** (`aleman_iuslex_ai`)[cite: 1, 2] para garantizar trazabilidad, versionado e integridad de hashes **SHA-256** en expedientes legales.

## Información general y despliegue

- **Repositorio:** `utm25090331-arch/AlemanIusLexAI`[cite: 3]
- **Programa (`program_id`):** `Chw3CkaxkRafhHTeJ85DQQJm4frpFwdhqCVtbkdZGtpF`
- **Red:** Solana Devnet[cite: 1, 2]
- **Entorno de desarrollo y pruebas:** Solana Playground (`beta.solpg.io`)[cite: 1, 2]

## Estructura del proyecto

```text
AlemanIusLexAI/
├── README.md
├── program/
│   └── lib.rs              # Programa Anchor en Rust (expedientes y registros SHA-256)
├── tests/
│   └── anchor.test.ts      # Suite de pruebas automatizadas (9/9 passing en Devnet)
└── frontend/
    └── index.html          # Interfaz web con Phantom, cálculo SHA-256 y derivación de PDAs
