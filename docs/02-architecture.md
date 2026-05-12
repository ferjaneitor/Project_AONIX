# 02 — Arquitectura por capas

## Vista general

AONIX está organizado en **15 capas funcionales**. Cada capa tiene una responsabilidad delimitada, una interfaz formal y dependencias explícitas. La separación es deliberada: cualquier módulo puede evolucionar, reescribirse o sustituirse sin invadir las responsabilidades de otros.

```
                    ┌──────────────────────────────────────────────┐
                    │      15. Coordinador central                 │
                    └──────────────────────────────────────────────┘
                              │
   ┌──────────────────────────┼─────────────────────────────────────┐
   │                          │                                     │
   ▼                          ▼                                     ▼
┌──────────────┐   ┌──────────────────┐                ┌───────────────────────┐
│ 11. Tradu-   │   │ 4. Validador     │                │ 14. Experimentación   │
│   cción      │   │ 5. Simulador     │                │   y auditoría         │
│   humana     │   │ 6. Verificador   │                └───────────────────────┘
├──────────────┤   │ 7. Evaluador     │
│ 12. Tradu-   │   │ 8. Pruebas       │                ┌───────────────────────┐
│   cción IA   │   └──────────────────┘                │ 13. Visualización 2D  │
└──────────────┘            │                          │   (Vulkan)            │
                            ▼                          └───────────────────────┘
                  ┌───────────────────┐                      ▲
                  │  1. Mundo lógico  │──────────────────────┘
                  └───────────────────┘
                            │
                            ▼
                  ┌───────────────────┐
                  │ 2. .aoncir        │
                  │ 3. .aonclg        │
                  └───────────────────┘
                            │
                            ▼
                  ┌───────────────────┐
                  │ 9. Memoria        │
                  │ 10. Currículo     │
                  └───────────────────┘
```

## Capa 1 — Mundo lógico

**Responsabilidad:** definir los tipos fundamentales del universo AONIX.

**Entidades:**

- `Signal` — identificador único de una señal lógica (entrada, salida, o intermedia).
- `Gate` — nodo lógico, exclusivamente uno de `{AND, OR, NOT}`.
- `Net` / `Wire` — conexión dirigida entre salida de un nodo y entrada de otro.
- `Circuit` — grafo dirigido acíclico (DAG) de gates y nets sobre un conjunto de señales.
- `Port` — entrada o salida externa del circuito, opcionalmente etiquetada semánticamente.
- `SemanticTag` — etiqueta sobre una señal o grupo (`carry`, `zero_flag`, `clock`, `bus`, etc.).
- `Spec` — especificación formal de comportamiento (tabla de verdad, propiedad lógica, modelo de referencia).
- `Task` — meta operativa: spec + nivel + restricciones + métricas.
- `Level` — peldaño curricular.
- `Test` — caso de prueba o suite proporcional al nivel.
- `EpisodeState` — estado de una sesión de construcción.

Esta capa **no contiene** lógica de simulación, validación ni visualización. Solo define las estructuras.

## Capa 2 — Representación canónica (`.aoncir`)

**Responsabilidad:** serializar y deserializar circuitos en su forma oficial activa.

Ver [03 — Formato `.aoncir`](03-format-aoncir.md) para la especificación completa.

Funciones clave:

- `parse(bytes) -> Result<Circuit>`
- `write(Circuit) -> bytes`
- `validate_structural(Circuit) -> Result<()>` — solo AND/OR/NOT, sin ciclos, sin señales colgantes.
- `hash_canonical(Circuit) -> Hash` — huella estable para deduplicar y comparar versiones.

## Capa 3 — Representación de aprendizaje (`.aonclg`)

**Responsabilidad:** serializar contexto de aprendizaje para IA sin contaminar la verdad técnica.

Ver [04 — Formato `.aonclg`](04-format-aonclg.md).

Es **separable** del `.aoncir`: un circuito puede tener un `.aoncir` sin `.aonclg`, pero todo `.aonclg` referencia un `.aoncir` existente.

## Capa 4 — Validador de acciones

**Responsabilidad:** decidir si una acción propuesta por un agente es legal **antes** de ejecutarla.

**Rechaza:**

- Compuertas no permitidas (todo lo que no sea AND/OR/NOT).
- Señales indefinidas o referenciadas antes de ser declaradas.
- Nombres duplicados de señal.
- Conexiones inválidas (tipo incompatible, aridad incorrecta).
- Ciclos en el grafo (a menos que el nivel lo permita explícitamente, p. ej. en estructuras de memoria con feedback gobernado por `clock`).
- Salidas asignadas desde señales inexistentes.
- Acciones incompatibles con el nivel actual.
- Estructuras que rompan la representación 2D.
- Intentos de escribir memoria sin validación previa.
- Intentos de declarar un circuito como correcto sin pasar por el verificador.

**Garantía:** el simulador, verificador y memoria solo reciben acciones que ya pasaron el validador.

## Capa 5 — Simulador

**Responsabilidad:** ejecutar un circuito sobre entradas concretas de forma **determinista**.

Modos:

1. **Por entrada específica** — un único vector de entrada, salida inmediata, ruta de señal trazable.
2. **Por lotes** — N vectores de entrada, salida agregada (matriz salida × entrada).
3. **Incremental** — recalcula solo los conos lógicos afectados por un cambio.
4. **Por cono lógico** — evalúa un subconjunto del circuito que alimenta una salida o región.
5. **Para visualización** — produce trazas anotadas: qué señales se activaron, qué compuertas dispararon, qué ruta llevó al resultado.
6. **Temporal** (futuro, niveles ≥ 11) — soporta señales etiquetadas como `clock`, `reset`, `enable` con avance discreto del tiempo.

**Garantía:** misma entrada + mismo circuito ⇒ mismo resultado, sin importar el momento, hilo o máquina.

## Capa 6 — Verificador

**Responsabilidad:** decidir si un circuito cumple su especificación.

Estrategias:

- Verificación exhaustiva cuando el espacio de entradas es viable (≤ 2^N para N ≤ ~20 con margen).
- Pruebas aleatorias reproducibles (semilla fija).
- Casos límite (todo-cero, todo-uno, alternados, un solo bit activo/apagado, históricos fallidos).
- Regresiones contra suite previa.
- Verificación por propiedades.
- Comparación contra modelo de referencia.
- Verificación modular por subcircuito.
- Validación incremental tras un cambio local.
- Validación por señal semántica (la salida etiquetada `carry` debe comportarse como acarreo).

El verificador es la **única fuente de la decisión "correcto/incorrecto"**.

## Capa 7 — Evaluador estructural

**Responsabilidad:** medir calidad, no verdad.

**No decide** si un circuito es correcto. Solo mide.

Métricas:

- Número de compuertas (total y por tipo).
- Profundidad lógica (longitud del camino crítico).
- Número de señales intermedias.
- Señales muertas (no alcanzables desde ninguna salida).
- Fan-in, fan-out por nodo.
- Rutas críticas.
- Reutilización (cuántas salidas comparten subexpresiones).
- Compartición entre salidas.
- Costo estructural agregado (ponderación configurable).
- Mejora contra versión histórica.
- Estabilidad bajo perturbación.
- Complejidad visual estimada.
- Costo de simulación.
- Costo de validación.

## Capa 8 — Pruebas escalables

**Responsabilidad:** definir suites de prueba proporcionales al nivel.

Ver [07 — Pruebas y verificación](07-testing-and-verification.md) para detalle.

Niveles de exigencia:

- **Lógica pequeña:** exhaustiva completa.
- **Mediana:** exhaustiva por submódulo + aleatorias con semilla + casos límite + históricos.
- **Grande:** dirigida + por propiedades + conos lógicos + comparación con referencia + regresión + diferencial + modular + incremental.
- **Arquitectónica:** secuencias temporales, ciclos de reloj, reset, enable, propagación de carry, validación de flags, programas pequeños, regresión sobre escenarios.

## Capa 9 — Memoria

**Responsabilidad:** persistencia técnica multi-rol.

Ver [05 — Sistema de memorias](05-memory-system.md) para los 10 tipos.

Reglas absolutas:

- Solo **una versión oficial activa** por circuito y tamaño.
- Las versiones anteriores se conservan en memoria histórica (no se borran).
- Memoria nunca puede usarse como atajo que viole las reglas absolutas.

## Capa 10 — Currículo

**Responsabilidad:** organizar el aprendizaje en niveles y definir cuándo se avanza.

Ver [06 — Sistema curricular](06-curriculum.md).

Niveles 0 a 13 mínimos; extensible a pipeline, hazards, caches, multinúcleo en visión futura.

## Capa 11 — Traducción humana

**Responsabilidad:** explicar al humano el qué, por qué y cómo del circuito y del proceso.

Genera explicaciones derivadas de la **estructura real**, nunca de especulación.

Explica:

- Qué hace un circuito.
- Por qué una salida se activa para una entrada dada.
- Qué señales son importantes (camino crítico, cuello de botella).
- Qué prueba falló y con qué entrada.
- Qué parte del circuito causa el error.
- Qué optimización se aplicó.
- Qué señales fueron eliminadas.
- Qué cambió entre versiones.
- Por qué una versión reemplazó a otra.
- Qué falta para avanzar de nivel.
- Qué significa una etiqueta semántica.
- Cómo se relaciona un circuito con una tarea.

## Capa 12 — Traducción para IA

**Responsabilidad:** convertir el mundo formal en estados, acciones, recompensas y contexto consumibles por modelos de IA.

Entrega:

- Tarea actual y nivel.
- Entradas, salidas, etiquetas.
- Circuito parcial y señales disponibles.
- Acciones legales en este punto (lista explícita, sin ambigüedad).
- Errores actuales y pruebas fallidas.
- Recompensa parcial.
- Historial de acciones del episodio.
- Estado visual abstracto.
- Métricas estructurales.
- Versiones históricas relevantes.
- Criterios de éxito.
- Restricciones absolutas (recordadas en cada estado).
- Progreso curricular.

La IA **no responde con texto libre ambiguo**. Responde con **acciones formales** que pasarán por el validador.

## Capa 13 — Visualización 2D (Vulkan)

**Responsabilidad:** renderizar el mundo formal en 2D de alto rendimiento.

**No decide, no verifica, no altera.** Solo visualiza.

Ver [09 — Visualización 2D con Vulkan](09-visualization-vulkan.md).

## Capa 14 — Experimentación y auditoría

**Responsabilidad:** trazabilidad completa.

Guarda:

- Intentos y resultados.
- Benchmarks.
- Comparaciones entre versiones.
- Regresiones detectadas.
- Optimizaciones aplicadas y revertidas.
- Histórico de decisiones del coordinador.

## Capa 15 — Coordinador central

**Responsabilidad:** orquestar todas las capas durante un episodio o tarea.

Ver [10 — Coordinador central](10-coordinator.md).

**No es una IA.** No inventa soluciones. Aplica reglas formales.

## Reglas de dependencia entre capas

1. Capas inferiores (Mundo lógico, formatos) **no dependen** de capas superiores.
2. El simulador no depende del verificador. El verificador llama al simulador.
3. El evaluador no decide; reporta al coordinador.
4. La visualización no escribe en memoria canónica.
5. La traducción para IA no puede generar acciones; solo describe estado y enumera acciones legales calculadas por el validador.
6. El coordinador es el único que puede mutar la memoria canónica, y solo a través del verificador.

## Implementación en Rust (mapeo previsto)

Workspace con crates separados por capa:

```
crates/
├── aonix-core/         # Capas 1, 2, 3 (modelo, parsers)
├── aonix-validate/     # Capa 4
├── aonix-sim/          # Capa 5
├── aonix-verify/       # Capa 6
├── aonix-eval/         # Capa 7
├── aonix-test/         # Capa 8
├── aonix-memory/       # Capa 9
├── aonix-curriculum/   # Capa 10
├── aonix-trans-human/  # Capa 11
├── aonix-trans-ai/     # Capa 12
├── aonix-vis/          # Capa 13 (Vulkan)
├── aonix-audit/        # Capa 14
├── aonix-coord/        # Capa 15
└── aonix-cli/          # Punto de entrada / interfaz
```

Esta partición se discutirá en [11 — Hoja de ruta](11-roadmap.md). No es obligatoria desde la fase 1; puede consolidarse a medida que crece el código.
