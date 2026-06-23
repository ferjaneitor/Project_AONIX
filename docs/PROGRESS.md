# AONIX — Registro de progreso (PROGRESS.md)

> **Léeme primero en cada sesión nueva.** Este archivo es el historial vivo del
> desarrollo: dónde se dejó el trabajo, qué está hecho y verificado, qué falta,
> y cómo comprobar el estado. Se actualiza con **cada modificación relevante**.
> La verdad normativa del *diseño* vive en `docs/00`–`docs/25`; este archivo es
> la verdad del *avance de implementación*.

---

## Estado actual (resumen)

- **Fase del roadmap:** Fases 1, 2 y 3 **COMPLETAS y verificadas**.
- **Estructura:** **workspace Cargo multi-crate** (7 crates).
- **Salud:** `cargo build` / `cargo test --workspace` / `cargo clippy --workspace --all-targets`
  / `cargo doc -D warnings` → **todo en verde**. **208 tests** pasando.
- **Reglas absolutas (R1 2D, R2 AND/OR/NOT):** respetadas y blindadas a nivel de tipos
  (y reforzadas en el validador de acciones).

### Cómo verificar (un solo bloque)

```bash
cargo build --workspace
cargo test  --workspace            # 208 tests, 0 fallos
cargo clippy --workspace --all-targets   # 0 warnings
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps   # 0 warnings
# CLI:
cargo run -q -p aonix-cli -- hash        crates/aonix/tests/data/one_bit_full_adder.aoncir
cargo run -q -p aonix-cli -- truth-table crates/aonix/tests/data/one_bit_full_adder.aoncir
```

---

## Mapa del workspace

```
Cargo.toml                      # workspace virtual (resolver 3, edición 2024)
crates/
├── aonix-core/                 # Capas 1–3: circuit_model + format/aoncir (lib aonix_core)
│   └── src/circuit_model, src/format/aoncir (parse, validate, write, hash, schema)
├── aonix-sim/                  # Capa 5: simulación (lib aonix_sim) → depende de aonix-core
│   └── src/simulation (evaluation, topological_order)
├── aonix-validate/             # Capa 4: validador de acciones (action, state, validate)
├── aonix-verify/               # Capa 6: verificador exhaustivo (spec, report, verify)
├── aonix-eval/                 # Capa 7: evaluador estructural (metrics, compare)
├── aonix/                      # Crate paraguas (facade): re-exporta circuit_model/format/simulation/validate/verify/eval
│   └── tests/ + tests/data/    # tests de integración + fixtures .aoncir
└── aonix-cli/                  # Binario `aonix` (CLI) → depende de la facade
```

Decisión de estructura: las capas 4, 6–15 tendrán su propio crate (`aonix-validate`,
`aonix-verify`, `aonix-eval`, `aonix-memory`, …) a medida que se implementen. La
facade `aonix` mantiene estables las rutas `aonix::circuit_model`, `aonix::format`,
`aonix::simulation` para tests y CLI.

---

## Estado por fase (roadmap docs/11)

| Fase | Nombre | Estado |
|------|--------|--------|
| 0 | Fundación documental | ✅ completa |
| 1 | Núcleo lógico mínimo | ✅ **completa y verificada** |
| 2 | Validador de acciones + verificador exhaustivo | ✅ **completa y verificada** |
| 3 | Evaluador estructural | ✅ **completa y verificada** |
| 4 | Memoria canónica e histórica | ⏳ **siguiente** (decisión pendiente: almacenamiento) |
| 5 | Pruebas escalables | ⬜ pendiente |
| 6 | Optimizador estructural | ⬜ pendiente |
| 7 | Currículo y tareas (0–5) | ⬜ pendiente |
| 8 | Traducción humana | ⬜ pendiente |
| 9 | Traducción para IA | ⬜ pendiente |
| 10 | Coordinador central | ⬜ pendiente |
| 11 | Visualización 2D Vulkan | ⬜ pendiente (decisión: backend; requiere GPU) |
| 12 | Niveles avanzados / ALU | ⬜ pendiente |
| 13 | Temporalidad / CPU mínima | ⬜ pendiente |
| 14 | Robustecimiento y benchmarks | ⬜ continua |

### Detalle Fase 1 (sub-fases)

| Sub-fase | Entregable | Estado |
|----------|-----------|--------|
| 1.A | Modelo de datos (circuit_model) | ✅ |
| 1.B | Parser `.aoncir` + schema | ✅ |
| 1.C | Validación documental | ✅ |
| 1.D | Writer canónico | ✅ |
| 1.E | Simulador vector único | ✅ |
| 1.F | Simulación por lotes + exhaustiva (tabla de verdad 2ⁿ) | ✅ |
| 1.J | Hash canónico `blake3:<hex>` + CLI | ✅ |

Los **3 criterios de aceptación de Fase 1** (roadmap) se cumplen: full adder simulado
sobre las 8 combinaciones da la tabla correcta; un nodo ≠ AND/OR/NOT falla el parser
con error explícito; mismo `.aoncir` ⇒ mismo hash canónico.

---

## Decisiones del usuario pendientes (bloquean fases concretas)

- **Backend Vulkan** (`ash` / `wgpu` / `vulkano`) — Fase 11. Requiere GPU/display.
- **Almacenamiento de memoria** (archivos planos vs SQLite/sled/redb) — Fase 4.
- **Licencia** del proyecto.

Decididas: **estructura = workspace multi-crate** (hecho); **cadencia = autónoma y
encadenada, con verificación exhaustiva y este historial por cada cambio**.

---

## Bitácora (entradas, más reciente arriba)

### 2026-06-23 — Fase 3: evaluador estructural
- **Crate `aonix-eval`** (capa 7): `Metrics` (conteo de compuertas por tipo,
  profundidad lógica / camino crítico, señales muertas, fan-in/out máximos,
  compartición de subexpresiones, costo agregado ponderado con `CostWeights`),
  `evaluate`/`evaluate_with_weights`, y comparador determinista `compare` /
  `default_compare` / `is_strictly_better` con `Criterion` y `DEFAULT_RANKING`
  (orden lexicográfico de docs/13 §28: conteo → profundidad → muertas → reuso;
  empate favorece al titular, docs/19). **El evaluador mide, no decide.**
- **Facade** `aonix` re-exporta `aonix::eval`.
- Fixture nuevo `one_bit_full_adder_redundant.aoncir` (full adder + 1 compuerta
  muerta) para el test del comparador.
- **Integración** (`crates/aonix/tests/phase3_evaluator.rs`): el full adder
  canónico mide 13 compuertas (6 AND/3 OR/4 NOT), profundidad 6, 0 muertas; la
  variante redundante mide 14 compuertas y 1 muerta; el comparador rankea la
  canónica como estrictamente mejor, de forma **estable y reproducible** entre
  corridas (criterio de aceptación de Fase 3).
- Binario renombrado a `aonix-cli` (evita colisión de docs con el crate facade
  `aonix`). Invocar: `cargo run -p aonix-cli -- <subcomando>`.
- +11 tests (de 197 a 208).

### 2026-06-23 — Fase 2: validador de acciones + verificador exhaustivo
- **Crate `aonix-validate`** (capa 4): `Action` (conjunto cerrado de acciones de
  construcción; `Action::create_gate` rechaza XOR/NAND/NOR/XNOR en la capa de
  acción), `BuildState` con las 10 reglas de docs/08 (`validate`/`apply`),
  `legal_action_kinds`, y `finalize` → `Circuit` (re-valida vía `CircuitBuilder`).
  `ValidationError` tipado (nivel L0 de docs/14).
- **Crate `aonix-verify`** (capa 6): `Specification` (`TruthTable` |
  `ReferenceFunction`), verificación **exhaustiva** PASA/FALLA con
  `VerificationReport` (casos evaluados + casos fallidos), reusando el simulador.
  Tope `MAX_EXHAUSTIVE_INPUT_BITS`. `VerifyError` para desajustes de aridad.
- **Facade** `aonix` ahora re-exporta `aonix::validate` y `aonix::verify`.
- **Integración** (`crates/aonix/tests/phase2_pipeline.rs`): se construye un half
  adder acción por acción sin rechazos, se finaliza y se verifica exhaustivamente
  (PASA); el full adder del fixture verifica contra función de referencia (PASA);
  un circuito incorrecto da FALLA con caso concreto; XOR se rechaza. Cumple los
  3 criterios de aceptación de Fase 2 del roadmap.
- Se añadió `Clone` a `AonixError` (todas sus variantes lo admiten).
- +23 tests (de 174 a 197). Nota: tras añadir un `derive`, `cargo doc` puede
  fallar por caché incremental; `cargo clean` lo resuelve.

### 2026-06-23 — Migración a workspace multi-crate
- Reestructurado el crate único en workspace: `aonix-core` (modelo + formato),
  `aonix-sim` (simulación), `aonix` (facade), `aonix-cli` (binario `aonix`).
- Imports del simulador actualizados a `aonix_core::circuit_model`. Enlaces de
  doc corregidos para `cargo doc -D warnings`.
- Tests movidos a `crates/aonix/tests/` (con sus fixtures). Verificado: 174 tests
  verdes, clippy y doc limpios, CLI operativa.

### 2026-06-23 — Cierre de Fase 1
- **Hash canónico** (`crates/aonix-core/src/format/aoncir/hash.rs`):
  `hash_canonical(&Circuit) → "blake3:<hex>"` sobre serialización determinista
  (puertos en orden, señales/gates/grupos canónicos, etiquetas), excluyendo
  meta/layout/verificación. Invariante al orden textual de `[[gates]]`.
- **Simulación por lotes y exhaustiva** (`aonix-sim`): `simulate_batch`,
  `simulate_exhaustive` (guarda `MAX_EXHAUSTIVE_INPUT_BITS = 20`), nueva variante
  `AonixError::ExhaustiveInputTooLarge`.
- **CLI** (`aonix-cli`): `validate | hash | canon | simulate | truth-table`.
- **Higiene:** fix clippy `explicit_auto_deref`; refuerzo C1 (entrada de gate que
  lee puerto de salida) en `CircuitBuilder::finish`; fixtures de déficit de aridad
  (`arity_and_one_input`, `arity_not_zero_inputs`).
- +19 tests (hash, batch/exhaustiva, parser, builder) → de 155 a 174.

---

## Siguiente paso concreto

**Fase 4 — Memoria canónica e histórica (crate `aonix-memory`):** persistencia con
políticas de reemplazo (docs/02 capa 9, docs/05, docs/19). Entregables:
1. Promoción **atómica** del oficial activo (una sola versión activa por circuito y
   tamaño), reutilizando `hash_canonical` (dedupe) e `is_strictly_better` (mejora
   estricta del evaluador).
2. Memoria histórica **append-only** (nunca se borra) + memoria experimental.
3. Índice + recuperación sin pérdida.

**Decisión pendiente del usuario — almacenamiento físico:** por defecto se propondrá
**archivos planos auditables** (TOML/JSON) + índice (recomendación del roadmap);
alternativa SQLite/sled/redb. Criterio de aceptación: promover a oficial activo es
transaccional; una versión "mejor" reemplaza a la activa y la anterior queda en el
histórico recuperable.
