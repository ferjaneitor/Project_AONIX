# 11 — Hoja de ruta por fases

## Filosofía del roadmap

AONIX es un sistema grande, pero **no se construye todo a la vez**. La hoja de ruta organiza el trabajo en fases con entregables verificables, cada una con valor independiente. Una fase termina cuando sus entregables pasan tests y son utilizables, no antes.

**Regla operativa:** ninguna fase introduce primitivas lógicas nuevas, jamás. Las reglas absolutas (R1, R2) son invariantes en todas las fases.

## Vista general

| Fase | Nombre | Duración relativa | Entregable principal |
|------|--------|-------------------|----------------------|
| 0 | Fundación documental | corta | Documentos canónicos + memoria persistente |
| 1 | Núcleo lógico mínimo | media | Modelo de datos + parser `.aoncir` + simulador determinista |
| 2 | Validador + verificador básico | media | Validación de acciones + verificación exhaustiva |
| 3 | Evaluador estructural | corta | Métricas básicas + ordenamiento canónico |
| 4 | Memoria canónica e histórica | media | Persistencia atómica + reemplazo de oficial activo |
| 5 | Pruebas escalables | media | Suites por nivel + casos límite + regresión |
| 6 | Optimizador estructural | larga | Transformaciones que preservan comportamiento |
| 7 | Currículo y tareas | media | Niveles 0–5 funcionales con tareas |
| 8 | Traducción humana | corta | Explicación generada desde estructura |
| 9 | Traducción para IA | media | Estado/acciones/recompensa serializables |
| 10 | Coordinador central | media | Ciclo de episodio integral |
| 11 | Visualización 2D Vulkan | larga | Render interactivo del grafo y simulación |
| 12 | Niveles avanzados | larga | Niveles 6–10 (buses, aritmética, ALU) |
| 13 | Temporalidad | larga | Clock, reset, enable; niveles 11–13 |
| 14 | Robustecimiento y benchmarks | continua | Profiling, optimización de rendimiento, suite de regresión completa |

Las fases pueden **solaparse parcialmente** si tienen dependencias compatibles. La visualización (Fase 11) es independiente del núcleo lógico tras la Fase 4 y puede empezar antes en paralelo si hay capacidad.

---

## Fase 0 — Fundación documental

**Estado:** **en curso** (este conjunto de documentos).

**Entregables:**

- `README.md` raíz.
- `docs/00-vision.md` a `docs/11-roadmap.md`.
- `docs/glossary.md`.
- Memoria persistente del proyecto (perfil del usuario, reglas absolutas, stack).
- Estructura inicial del workspace Cargo.

**Criterio de cierre:** el usuario revisa la documentación y confirma alineación con su visión.

**No incluye:** código de simulación/verificación.

---

## Fase 1 — Núcleo lógico mínimo

**Objetivo:** modelo de datos y simulación determinista.

**Entregables:**

1. `crate aonix-core`:
   - Tipos `Signal`, `Gate (AND|OR|NOT)`, `Net`, `Circuit`, `Port`, `SemanticTag`.
   - DAG inmutable representado en memoria.
   - Ordenamiento topológico determinista.
2. Parser y writer de `.aoncir` (representación textual inicial, TOML-like).
3. Validación estructural del parser:
   - Tipo de nodo ∈ {AND, OR, NOT}.
   - Aridad correcta.
   - Sin señales colgantes.
   - DAG (sin ciclos).
   - Nombres únicos.
4. Simulador determinista:
   - Modo "una entrada".
   - Modo "lote".
5. Hash canónico estable.
6. Suite de tests unitarios para todo lo anterior.

**Criterios de aceptación:**

- Cargar un `.aoncir` válido de un `one_bit_full_adder` y simularlo sobre las 8 combinaciones produce la tabla de verdad correcta.
- Cualquier `.aoncir` con un nodo distinto a AND/OR/NOT falla el parser con error explícito.
- Mismo `.aoncir` ⇒ mismo hash canónico.

**No incluye:** verificador completo, optimizador, memoria persistente.

---

## Fase 2 — Validador + verificador básico

**Objetivo:** acciones legales y verificación exhaustiva.

**Entregables:**

1. `crate aonix-validate`:
   - Lista enumerable de acciones permitidas.
   - Reglas de validación (ver [08 — Acciones](08-actions-and-rewards.md)).
   - Cálculo de acciones legales dado un estado.
2. `crate aonix-verify` (versión inicial):
   - Verificación exhaustiva para entradas ≤ N (configurable, por defecto N ≤ 12).
   - Decisión binaria PASA/FALLA.
   - Reporte estructurado de casos fallidos.
3. Integración con `aonix-core` (parser ⇒ validador ⇒ simulador ⇒ verificador).

**Criterios de aceptación:**

- Construir un `four_bit_full_adder` paso a paso, acción por acción, sin que el validador rechace nada legítimo.
- Verificación exhaustiva del `four_bit_full_adder` confirma correctitud o detecta fallo con caso específico.
- Cualquier acción que intente introducir XOR/NAND/NOR/XNOR como primitiva es rechazada.

---

## Fase 3 — Evaluador estructural

**Objetivo:** métricas que ordenen versiones.

**Entregables:**

1. `crate aonix-eval`:
   - Conteo de compuertas por tipo.
   - Profundidad lógica.
   - Señales muertas.
   - Fan-in / fan-out.
   - Reutilización (estimación de compartición entre salidas).
   - Costo agregado (función ponderada configurable).
2. Comparador entre dos circuitos (ranking lexicográfico configurable).

**Criterios de aceptación:**

- Dadas dos versiones de `one_bit_full_adder`, el evaluador devuelve un orden estable y reproducible.

---

## Fase 4 — Memoria canónica e histórica

**Objetivo:** persistencia con políticas de reemplazo.

**Entregables:**

1. `crate aonix-memory`:
   - Sistema de archivos plano + índice (decisión sobre DB embebida queda abierta).
   - Operación de promoción atómica (oficial activo).
   - Memoria histórica append-only.
   - Memoria experimental para descartes.
2. CLI mínima para inspeccionar memoria (`aonix mem list`, `aonix mem show <name>`).

**Criterios de aceptación:**

- Promover un `.aoncir` a oficial activo es transaccional: o se completa íntegro o no se completa.
- Una versión "mejor" detectada por evaluador reemplaza correctamente la oficial activa; la anterior queda en histórico.
- Recuperar la histórica funciona sin pérdida.

---

## Fase 5 — Pruebas escalables

**Objetivo:** suites por nivel, casos límite, regresión.

**Entregables:**

1. `crate aonix-test`:
   - Suite exhaustiva.
   - Suite aleatoria con semilla.
   - Catálogo de casos límite.
   - Suite de regresión por circuito y por familia.
2. Integración con `aonix-verify`.

**Criterios de aceptación:**

- Para `eight_bit_full_adder`, el verificador combina exhaustiva submódulo + aleatoria + casos límite + regresión.
- Un caso histórico fallido reaparece en futuras verificaciones automáticamente.

---

## Fase 6 — Optimizador estructural

**Objetivo:** mejorar circuitos preservando comportamiento.

**Entregables:**

1. `crate aonix-opt`:
   - Eliminación de señales muertas.
   - Eliminación de compuertas redundantes.
   - Detección de subexpresiones comunes y reutilización.
   - Aplicación de leyes booleanas (De Morgan, absorción, idempotencia, doble negación).
   - Reducción de profundidad por reasociación.
   - Iteración hasta punto fijo.
2. Verificación post-optimización (cada paso re-verifica).
3. Registro de transformaciones aplicadas (memoria de optimización).

**Criterios de aceptación:**

- Un `four_bit_full_adder` con redundancia conocida se reduce al canónico mínimo (o al menos a una mejora medible).
- Ninguna optimización publicada falla pruebas que la versión pre-optimizada superaba.

---

## Fase 7 — Currículo y tareas (niveles 0–5)

**Objetivo:** sistema de tareas operativo en niveles iniciales.

**Entregables:**

1. `crate aonix-curriculum`:
   - Definición formal de `Task`, `Level`, `Progress`.
   - Niveles 0 a 5 con sus tareas catalogadas.
   - Condiciones de avance medibles.
2. Persistencia de progreso curricular por agente.

**Criterios de aceptación:**

- Un agente humano puede recorrer niveles 0–5 vía CLI proponiendo acciones y recibiendo retroalimentación.

---

## Fase 8 — Traducción humana

**Objetivo:** explicaciones derivadas de la estructura.

**Entregables:**

1. `crate aonix-trans-human`:
   - Explicación de qué hace un circuito (descripción funcional desde tabla de verdad reducida).
   - Explicación de por qué una salida se activa para una entrada (camino de propagación).
   - Explicación de error (caso fallido + región responsable).
   - Explicación de optimización aplicada.
   - Explicación de reemplazo de versión.

**Criterios de aceptación:**

- Para cualquier `.aoncir` válido, el traductor genera un párrafo legible que describe correctamente su comportamiento.

---

## Fase 9 — Traducción para IA

**Objetivo:** entrega estructurada de estado y acciones legales.

**Entregables:**

1. `crate aonix-trans-ai`:
   - Serialización del estado del episodio (parcial + completo).
   - Lista enumerable de acciones legales.
   - Serialización de recompensa parcial.
   - Hooks de captura de trayectoria para `.aonclg`.

**Criterios de aceptación:**

- Una IA externa puede consumir el estado, elegir una acción legal de la lista, y el coordinador la procesa correctamente.
- La trayectoria capturada es suficiente para reconstruir el episodio bit a bit.

---

## Fase 10 — Coordinador central

**Objetivo:** ciclo completo de episodio integrado.

**Entregables:**

1. `crate aonix-coord`:
   - Implementación del ciclo descrito en [10 — Coordinador](10-coordinator.md).
   - Soporte para múltiples agentes (humano CLI, búsqueda exhaustiva baseline, agente externo conectado).
   - Cierre limpio con persistencia completa.

**Criterios de aceptación:**

- Episodio completo end-to-end sobre `four_bit_full_adder` con un agente "búsqueda exhaustiva pequeña": tarea → construcción → verificación → optimización → comparación → promoción → `.aonclg` → cierre.
- Episodios concurrentes funcionan sin condición de carrera en memoria canónica.

---

## Fase 11 — Visualización 2D Vulkan

**Objetivo:** render interactivo del grafo.

**Entregables:**

1. `crate aonix-vis`:
   - Pipeline Vulkan operativo.
   - Render de grafo con AND/OR/NOT, señales, puertos.
   - Layout layered determinista.
   - Resaltado de conos lógicos.
   - Animación de simulación de una entrada.
   - Comparación antes/después.
2. CLI/GUI mínima para abrir un `.aoncir` y navegarlo.

**Criterios de aceptación:**

- Abrir un `four_bit_full_adder.aoncir` y ver su grafo con interacción fluida.
- Ejecutar una entrada específica y ver el flujo de señal animado.
- 60 FPS en grafos de hasta 10³ nodos.

**Nota:** esta fase puede empezar en paralelo a Fase 5 o posteriores, una vez que `aonix-core` esté estable.

---

## Fase 12 — Niveles avanzados (6–10)

**Objetivo:** buses, aritmética, ALU mínima.

**Entregables:**

- Tareas y suites de niveles 6, 7, 8, 9, 10.
- Catálogo de circuitos compuestos asociados:
  - Mux/demux con buses.
  - Sumadores y restadores 4/8/16/32 bits.
  - Comparadores N-bit.
  - Unidades de flags.
  - ALU 4-bit.

---

## Fase 13 — Temporalidad y niveles 11–13

**Objetivo:** clock, reset, enable; latches, flip-flops, registros, memoria, CPU mínima.

**Entregables:**

- Extensión del simulador a modo temporal discreto.
- Soporte de ciclos controlados por `clock`.
- Verificación temporal.
- Tareas de niveles 11, 12, 13.
- CPU mínima de ejemplo (`minimal_cpu.aoncir`).

---

## Fase 14 — Robustecimiento y benchmarks

**Objetivo:** calidad de producción.

Tarea continua. Incluye:

- Profiling y optimización de rendimiento del simulador, verificador, optimizador.
- Suite de regresión completa.
- Fuzzing del parser.
- Documentación de cada módulo.
- API estable.
- Benchmarks comparativos.

---

## Decisiones pendientes que afectan al roadmap

A negociar con el usuario antes de iniciar Fase 1:

1. **Representación física de `.aoncir` / `.aonclg`:** texto (TOML, propio DSL), binario, o híbrido.
2. **Almacenamiento de memoria:** archivos planos solos, archivos planos + SQLite/sled/redb, o esquema propio.
3. **Backend Vulkan:** `ash` (bajo nivel) vs `wgpu` (más abstracción) vs `vulkano`.
4. **Estructura Cargo:** monolítico (un crate) inicial vs workspace multi-crate desde el principio.
5. **Política de versionado de AONIX:** semver completo o calendario.
6. **CI/CD:** GitHub Actions u otro; tests obligatorios antes de merge.
7. **Licencia.**

## Ordenamiento recomendado

Dependencias estrictas:

```
Fase 0 → Fase 1 → Fase 2 → Fase 3 → Fase 4
                              │
                              ├── Fase 5 → Fase 6 → Fase 7
                              │                       │
                              │                       └── Fase 8, 9 → Fase 10
                              │                                          │
                              └── Fase 11 (en paralelo desde Fase 4)     │
                                                                         ▼
                                                                    Fase 12, 13, 14
```

Fase 11 (visualización) puede comenzar en paralelo con Fase 5 si hay capacidad. Las demás siguen el orden lineal sugerido.

## Lo que NO está en el roadmap

- Compuertas derivadas como primitivas. **Nunca.**
- Una IA propia integrada a AONIX. AONIX es el entorno; la IA es externa.
- Síntesis automática que sustituya el aprendizaje (AONIX puede usar búsqueda exhaustiva como agente baseline, pero el propósito de la plataforma es ser entorno de aprendizaje, no fábrica de circuitos).
