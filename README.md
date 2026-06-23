# AONIX

**AND-OR-NOT Integrated eXploration**

AONIX es una plataforma determinista, 2D, escalable, verificable y visual donde agentes humanos, buscadores automáticos y modelos de IA aprenden a construir, simular, verificar, optimizar, visualizar y explicar circuitos digitales usando exclusivamente las compuertas primitivas **AND**, **OR** y **NOT**.

> La IA propone, pero AONIX determina la verdad técnica.

AONIX no es la IA. AONIX es el mundo formal donde la IA aprende.

---

## Reglas absolutas

1. El sistema es un entorno lógico y visual **2D**.
2. Las únicas compuertas primitivas son **AND, OR y NOT**.

No existen como primitivas: XOR, XNOR, NAND, NOR, ni ninguna otra derivada. Los circuitos compuestos (multiplexores, full adders, ALUs, registros, CPUs mínimas) **sí** pueden guardarse como entidades canónicas, siempre que estén expandidos internamente a AND/OR/NOT.

Detalle exhaustivo: [docs/01-rules-absolute.md](docs/01-rules-absolute.md).

---

## Mapa de capas

| # | Capa | Responsabilidad |
|---|------|-----------------|
| 1 | Mundo lógico | Señales, compuertas, circuitos, estados, tareas, pruebas, niveles |
| 2 | `.aoncir` | Representación canónica del circuito |
| 3 | `.aonclg` | Representación de aprendizaje para IA |
| 4 | Validador | Revisa si una acción es legal antes de ejecutarla |
| 5 | Simulador | Ejecuta circuitos sobre entradas |
| 6 | Verificador | Decide si el circuito cumple su especificación |
| 7 | Evaluador | Mide calidad estructural |
| 8 | Pruebas escalables | Exhaustivas, aleatorias, dirigidas, por propiedades, modulares |
| 9 | Memoria | Canónica, histórica, aprendizaje, experimental, pruebas, visual, curricular, trayectorias, fallos, optimización |
| 10 | Currículo | Niveles y condiciones de avance |
| 11 | Traducción humana | Explica circuitos, errores, métricas, decisiones |
| 12 | Traducción para IA | Estado, acciones, recompensas, contexto |
| 13 | Visualización 2D (Vulkan) | Renderiza grafos, señales, conos, flujos |
| 14 | Experimentación y auditoría | Intentos, regresiones, benchmarks, trazabilidad |
| 15 | Coordinador central | Orquesta todos los módulos por episodio |

Detalle por capa: [docs/02-architecture.md](docs/02-architecture.md).

---

## Documentación

Núcleo:

- [00 — Visión y principio rector](docs/00-vision.md)
- [01 — Reglas absolutas](docs/01-rules-absolute.md)
- [02 — Arquitectura por capas](docs/02-architecture.md)

Formatos canónicos:

- [03 — Formato `.aoncir`](docs/03-format-aoncir.md)
- [04 — Formato `.aonclg`](docs/04-format-aonclg.md)
- [17 — Relación formal entre `.aoncir` y `.aonclg`](docs/17-aoncir-aonclg-relationship.md)

Subsistemas:

- [05 — Sistema de memorias](docs/05-memory-system.md)
- [06 — Sistema curricular](docs/06-curriculum.md)
- [07 — Pruebas y verificación](docs/07-testing-and-verification.md)
- [08 — Acciones y función de recompensa](docs/08-actions-and-rewards.md)
- [09 — Visualización 2D con Vulkan](docs/09-visualization-vulkan.md)
- [10 — Coordinador central](docs/10-coordinator.md)

Especificaciones formales (consolidación):

- [12 — Especificación formal de tareas](docs/12-task-specification.md)
- [13 — Reglas de aceptación de un circuito](docs/13-circuit-acceptance.md)
- [14 — Reglas de rechazo](docs/14-circuit-rejection.md)
- [15 — Reglas de optimización](docs/15-optimization-rules.md)
- [16 — Límites de visibilidad para la IA](docs/16-ai-visibility-limits.md)
- [18 — Memoria operativa vs no operativa](docs/18-operational-vs-non-operational-memory.md)
- [19 — Política de versionado (oficial activo vs histórica)](docs/19-versioning-policy.md)

Cierre normativo concreto (catálogos y políticas):

- [20 — Catálogo inicial de tareas (niveles 0–5)](docs/20-task-catalog-levels-0-5.md)
- [21 — Sintaxis física de `.aoncir` (TOML inicial)](docs/21-aoncir-syntax.md)
- [22 — Catálogo de casos límite por familia](docs/22-edge-case-catalog.md)
- [23 — Catálogo de transformaciones del optimizador](docs/23-optimizer-transformations.md)
- [24 — Convenciones de etiquetas semánticas](docs/24-semantic-tag-conventions.md)
- [25 — Política de auditoría humana](docs/25-human-audit-policy.md)

Operativa:

- [Registro de progreso (PROGRESS)](docs/PROGRESS.md)
- [11 — Hoja de ruta por fases](docs/11-roadmap.md)
- [Glosario AONIX (normativo)](docs/glossary.md)

---

## Estado

**Fase 1 (Núcleo lógico mínimo) completa y verificada.** Implementados el modelo de circuito, el parser/writer `.aoncir`, la validación, el simulador determinista (vector único, por lotes y tabla de verdad exhaustiva), el **hash canónico** `blake3:` y una **CLI**. El código está organizado como **workspace multi-crate** (`aonix-core`, `aonix-sim`, `aonix`, `aonix-cli`).

El registro de avance vivo está en [docs/PROGRESS.md](docs/PROGRESS.md); las fases siguientes, en [docs/11-roadmap.md](docs/11-roadmap.md).

Verificación: `cargo test --workspace` (174 tests), `cargo clippy --workspace --all-targets` y `cargo doc` sin warnings.

## Stack

- **Lenguaje:** Rust edición 2024
- **Visualización:** Vulkan (capa puramente de render; no decide, no verifica)
- **Formatos propios:** `.aoncir`, `.aonclg`
- **Determinismo:** misma entrada + mismo circuito → mismo resultado, siempre

## Licencia

Por definir.
