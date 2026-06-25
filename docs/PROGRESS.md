# AONIX — Registro de progreso (PROGRESS.md)

> **Léeme primero en cada sesión nueva.** Este archivo es el historial vivo del
> desarrollo: dónde se dejó el trabajo, qué está hecho y verificado, qué falta,
> y cómo comprobar el estado. Se actualiza con **cada modificación relevante**.
> La verdad normativa del *diseño* vive en `docs/00`–`docs/25`; este archivo es
> la verdad del *avance de implementación*.

---

## Estado actual (resumen)

- **Fase del roadmap:** Fases 1, 2, 3, 4 y 5 **COMPLETAS y verificadas**.
- **Estructura:** **workspace Cargo multi-crate** (9 crates).
- **Salud:** `cargo build` / `cargo test --workspace` / `cargo clippy --workspace --all-targets`
  / `cargo doc -D warnings` → **todo en verde**. **226 tests** pasando.
- **Reglas absolutas (R1 2D, R2 AND/OR/NOT):** respetadas y blindadas a nivel de tipos
  (y reforzadas en el validador de acciones).

### Cómo verificar (un solo bloque)

```bash
cargo build --workspace
cargo test  --workspace            # 226 tests, 0 fallos
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
├── aonix-memory/               # Capa 9: memoria canónica/histórica flat-file (store)
├── aonix-test/                 # Capa 8: pruebas escalables (prng, generators, suite)
├── aonix/                      # Crate paraguas (facade): re-exporta circuit_model/format/simulation/validate/verify/eval/memory/testing
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
| 4 | Memoria canónica e histórica | ✅ **completa y verificada** (flat-file) |
| 5 | Pruebas escalables | ✅ **completa y verificada** |
| 6 | Optimizador estructural | ⏳ **siguiente** |
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

### 2026-06-24 — Fase 5: pruebas escalables (+ mejoras a lo existente)
- **Crate `aonix-test`** (capa 8): PRNG determinista propio `SplitMix64` (semilla
  explícita), generadores `exhaustive`/`random`/`edge_cases`, `RegressionSuite`
  append-only, y `run_suite` que combina suites contra una `Specification` y entrega
  PASA/FALLA agregado con desglose por suite. El runner elige qué entradas; el
  verificador decide.
- **Mejora a `aonix-verify`:** nuevo primitivo componible `verify_inputs(circuit,
  spec, &[inputs])` + accesores `Specification::{input_arity,output_arity,
  expected_output}`. Lo reusa `aonix-test`.
- **Mejora a `aonix-eval`:** nuevo `Criterion::FanOutMax` (menor es mejor), alineado
  con el `ranking_order` por defecto de docs/19; `DEFAULT_RANKING` sin cambios.
- **Facade** `aonix` re-exporta `aonix::testing` (no `aonix::test`, para no chocar con
  el namespace de tests).
- **Integración** (`crates/aonix/tests/phase5_testing.rs`): suite combinada PASA sobre
  AND y full adder; FALLA con casos concretos sobre spec equivocada; **un caso de
  regresión registrado reaparece automáticamente** en runs posteriores; aleatoria
  reproducible con misma semilla. Cumple el criterio de aceptación de Fase 5.
- +11 tests (de 215 a 226).

### 2026-06-24 — Fase 4: memoria canónica e histórica
- **Crate `aonix-memory`** (capa 9), almacenamiento **flat-file** (decisión por
  defecto). `MemoryStore` con layout `<root>/<name>/<parameters>/{active.aoncir,
  history/<hex>.aoncir, experimental/<hex>.aoncir}`. `CircuitKey` (name+parameters,
  validado [a-z0-9_]). `promote`/`promote_with_ranking` → `PromotionOutcome`
  (InstalledFirst | Replaced | AlreadyActive | RejectedNotBetter).
- Semántica docs/19: **1 oficial activo por (name,parameters)**; promoción **atómica**
  (archiva titular append-only, luego rename atómico del active); reemplazo solo por
  **mejora estricta** (reusa `aonix_eval::is_strictly_better`); **dedupe por hash**
  canónico; histórico recuperable sin pérdida.
- **CLI** `aonix-cli mem <list|show|history|promote>`.
- **Integración** (`crates/aonix/tests/phase4_memory.rs`): ciclo completo con el full
  adder canónico vs redundante — instalar (peor) → reemplazar (mejor, archiva) →
  dedupe → rechazar (peor) a experimental → recuperar histórico sin pérdida. Cumple
  los 3 criterios de aceptación de Fase 4.
- +7 tests (de 208 a 215).
- Nota: `cargo doc` puede fallar con "unresolved import" tras añadir un módulo a la
  facade por fingerprint obsoleto; `cargo clean` completo (no solo `--doc`) lo resuelve.

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

**Fase 6 — Optimizador estructural (crate `aonix-opt`):** transformaciones que
**preservan comportamiento** (docs/15, docs/23). Entregables:
1. Transformaciones legítimas: eliminación de señales muertas, doble negación
   (`NOT NOT x → x`), compuertas redundantes (`AND(x,x)→x`, etc.), CSE/compartición.
2. Iteración hasta **punto fijo** (o tope), cada paso atómico, con delta de métricas
   vía `aonix-eval`.
3. **Doble garantía** de preservación: vía algebraica (catálogo) **+** re-verificación
   completa tras optimizar (reusa `aonix-verify`/`aonix-test`); si regresa, se descarta
   esa transformación (backtracking), no el resto.

**Prohibido (docs/23 P.1–P.7):** nunca colapsar un patrón en nodo XOR/NAND/NOR/XNOR ni
en subcircuito opaco; nunca saltar la re-verificación. Criterio de aceptación: un
circuito con redundancia conocida se reduce a una mejora medible sin fallar ninguna
prueba que el original superaba.
