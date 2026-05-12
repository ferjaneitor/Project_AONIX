# 19 — Política de versionado: oficial activo vs versiones históricas

> **Documento normativo.** Define qué es una versión, cuántas pueden existir, cómo se promueve una nueva, cómo se archiva la anterior, qué garantías de inmutabilidad rigen sobre cada estado, y cómo se gestionan colisiones, regresiones y conflictos de concurrencia.

## Principio

> **Una sola verdad oficial activa, historial completo.**

AONIX tiene exactamente **una** versión oficial activa por (circuito + tamaño). Esto representa la mejor verdad técnica vigente. AONIX también conserva **todas** las versiones previamente verificadas como memoria histórica append-only. La verdad oficial cambia con disciplina; la historia no se pierde.

## Conceptos

### Circuito

Una entidad lógica con nombre canónico (`one_bit_full_adder`, `four_bit_comparator`, `minimal_arithmetic_logic_unit`, etc.) y, cuando aplique, un tamaño paramétrico (`width` u otros parámetros).

### Identidad de versión

Una versión concreta queda identificada por:

```
VersionId = (name, parameters, hash_canonical)
```

Donde `parameters` puede incluir `width` y otros parámetros estructurales si los hay.

Dos `.aoncir` con el mismo `hash_canonical` son **estructuralmente equivalentes**. La equivalencia de comportamiento se verifica independientemente, pero la equivalencia estructural se trata como **identidad**: AONIX no archiva dos veces el mismo circuito estructural.

### Oficial activo

Para cada `(name, parameters)` existe **a lo sumo una** versión oficial activa en memoria canónica. Es la versión:

- Más recientemente verificada.
- Estrictamente mejor que todas las versiones que la han precedido (según el `ranking_order` aplicable cuando fue promovida).
- Servida por defecto cuando alguien consulta el circuito por nombre.

### Versión histórica

Una versión previamente oficial activa o una versión verificada conservada explícitamente para comparación. Vive en memoria histórica append-only. Nunca se reactiva sin pasar de nuevo por el flujo de verificación, evaluación y promoción.

### Versión experimental

Una versión candidata que fue verificada pero **no promovida** (por L2 — evaluador no acepta, o L4 — coordinador no acepta). Vive en memoria experimental. **No es** versión histórica oficial; es trazabilidad de intento.

## Reglas absolutas de versionado

### V.1 — Unicidad del oficial activo

Para cualquier `(name, parameters)`, existe **0 o 1** versión oficial activa. Nunca 2 o más simultáneas. La transacción de promoción garantiza esto.

### V.2 — Append-only del historial

Las versiones históricas **nunca se borran ni modifican**. Se añaden. Una versión que entra al historial permanece accesible para siempre (salvo operación administrativa explícita, excepcional y auditada).

### V.3 — Promoción atómica

Cambiar el oficial activo es una **transacción atómica** que incluye:

1. Verificar que la nueva versión pasa todas las puertas de aceptación (ver [13](13-circuit-acceptance.md)).
2. Verificar que mejora estrictamente al incumbente bajo el `ranking_order` aplicable (ver más abajo).
3. Mover la versión incumbente a memoria histórica con `archived_at` y `superseded_by`.
4. Escribir la nueva versión como oficial activa.
5. Actualizar índices.
6. Registrar entrada en memoria de optimización con delta.
7. Registrar entrada en `.aonclg` del episodio que la produjo.

O ocurren todos los pasos íntegros, o no ocurre ninguno. Sin estados intermedios visibles.

### V.4 — Imposibilidad de degradar el oficial activo

Una versión nueva **nunca** reemplaza al oficial activo si:

- No pasa la verificación funcional (regresión).
- Empata según el `ranking_order` aplicable (estabilidad del incumbente).
- Es estrictamente peor en alguna dimensión dominante.

El criterio "mejor" es **estricto** y bajo orden lexicográfico del `ranking_order` declarado por la tarea (defecto: `gate_count, depth, dead_signals, fan_out_max, complexity_visual`).

### V.5 — Ningún ciclo automático

AONIX **no revierte automáticamente** al histórico salvo en escenarios bien definidos:

- **Detección de regresión tardía**: si el oficial activo falla un caso recién añadido al catálogo de pruebas y se determina que un histórico anterior lo superaba, el oficial activo se marca como **degradado** y el coordinador inicia un proceso de revisión humana. La reversión efectiva al histórico se hace solo tras intervención auditada.
- **Recuperación de fallo administrativo**: si un acto administrativo erróneo elimina o corrompe un oficial activo, la última versión histórica válida puede restaurarse como oficial activo. Esto es operación excepcional con auditoría completa.

En ningún otro caso AONIX revierte automáticamente.

### V.6 — Trazabilidad completa

Toda versión (activa, histórica o experimental) registra:

- `hash_canonical`.
- `predecessor: hash_canonical?` — versión que reemplazó al promoverse, si la había.
- `successor: hash_canonical?` — versión que la reemplazó después, si fue reemplazada.
- `created_at`, `verified_at`, `archived_at?`.
- `verification_report` — qué suite, cuántos casos, semilla, resultados.
- `evaluation_snapshot` — métricas en el momento de la promoción.
- `episode_origin: aonclg_id?` — `.aonclg` del episodio que la produjo (puede ser null para versiones importadas o creadas por búsqueda exhaustiva sin agente).
- `agent: AgentId` — quién la produjo.

La cadena `predecessor`/`successor` forma una **lista enlazada** lineal por circuito+parámetros. No hay ramas paralelas en memoria canónica.

### V.7 — Versiones paramétricas son independientes

`one_bit_full_adder.aoncir` y `four_bit_full_adder.aoncir` son **circuitos distintos** desde el punto de vista de la regla "un oficial activo". Cada `(name, width)` tiene su propio oficial activo. Pueden existir simultáneamente:

```
Oficial activo:
  one_bit_full_adder.aoncir         (width = 1)
  two_bit_full_adder.aoncir         (width = 2)
  four_bit_full_adder.aoncir        (width = 4)
  eight_bit_full_adder.aoncir       (width = 8)
  sixteen_bit_full_adder.aoncir     (width = 16)
  thirty_two_bit_full_adder.aoncir  (width = 32)
```

Cada uno con su propio historial.

### V.8 — Coherencia de la familia

Aunque cada `(name, parameters)` tiene su versión activa independiente, AONIX rastrea la **familia** (`name` sin parámetros). La familia comparte:

- Suite de regresión común (casos descubiertos en cualquier `width` pueden re-evaluarse en otros si aplican).
- Estilo de etiquetas semánticas (un `carry` debe llamarse `carry` en toda la familia).
- Modelos de referencia compatibles (un sumador aritmético en software cubre todos los `width`).

Una incoherencia detectada (p.ej. la versión `width=8` usa `carry_out` y la `width=16` usa `cout`) se registra como advertencia para revisión.

## Mecánica de la promoción

```
candidato = circuito producido por episodio
        │
        ▼
verificador entrega PASA sobre todas las suites de la tarea? ─ NO ─► descartar (L1)
        │ SÍ
        ▼
optimizador aplica transformaciones del catálogo
        │
        ▼
re-verificador entrega PASA sobre todas las suites? ─ NO ─► retroceder transformación,
        │ SÍ                                              repetir hasta versión verificada
        ▼
existe oficial activo previo para (name, parameters)?
        │
   ┌────┴────┐
   │         │
   NO        SÍ
   │         │
   ▼         ▼
promoción ranking_order(candidato, incumbente):
directa     estricto mejor? ─ NO ─► no promoción (L2),
   │           │                    candidato → memoria experimental
   │           SÍ
   │           │
   │           ▼
   │       comprobar concurrencia:
   │       ¿el oficial activo cambió entre el inicio del episodio
   │        y este momento?
   │           │
   │       ┌───┴───┐
   │       │       │
   │       NO      SÍ
   │       │       │
   │       │       ▼
   │       │   re-evaluar candidato contra el NUEVO oficial activo
   │       │   (vuelve a "ranking_order")
   │       │
   │       ▼
   ▼   transacción atómica:
   ▼     - mover incumbente a histórica (predecessor link)
   ▼     - escribir candidato como oficial activo
   ▼     - vincular successor en el histórico
   ▼     - actualizar índices
   ▼     - registrar memoria de optimización con delta
   ▼     - actualizar .aonclg con bandera promoted=true
   ▼     - cerrar episodio
   ▼
   completar transacción
```

## Reglas de ranking y mejora estricta

El `ranking_order` de la tarea define el orden lexicográfico de comparación:

```
ranking_order: [gate_count, depth, dead_signals, fan_out_max, ...]
```

**Mejora estricta** significa que el candidato es estrictamente menor (mejor) en al menos una dimensión sin ser peor en dimensiones más importantes (más a la izquierda en el orden lexicográfico).

**Empate** significa que candidato e incumbente coinciden en todas las dimensiones del `ranking_order`. En empate, **gana el incumbente** por estabilidad: no se reemplaza.

**Tolerancia epsilon**: para evitar ruido o iteraciones triviales que cambian el grafo sin valor real, la tarea puede declarar un epsilon mínimo de mejora. Mejoras por debajo del epsilon se consideran empate.

## Conflictos de concurrencia

Cuando múltiples episodios resuelven la misma tarea en paralelo:

### Escenario A — promociones secuenciales

Episodio E1 cierra antes que E2. E1 promueve su versión a oficial activo. E2 cierra después; antes de promover, AONIX detecta que el oficial activo cambió y re-evalúa el candidato de E2 contra el **nuevo** oficial activo (la versión de E1). Si E2 lo supera estrictamente, promueve sobre él (la versión de E1 pasa a histórica). Si no, E2 queda como experimental.

### Escenario B — promociones simultáneas

Dos episodios intentan promover "al mismo tiempo". La transacción atómica garantiza que solo **una** se completa primero. La otra se revalúa contra el nuevo oficial activo. La operación se serializa por transacción.

### Escenario C — promoción durante un episodio en curso

E1 está en curso. E2 promueve. Cuando E1 cierra, su candidato se compara contra el nuevo oficial activo (versión de E2). El `.aonclg` de E1 registra que el oficial activo cambió entre el inicio y el cierre.

## Reversión y desactivación

### Reversión administrativa

Si por error se promovió una versión defectuosa que el verificador no debió aprobar (bug del propio AONIX), procede:

1. Análisis del fallo: identificar por qué se aprobó.
2. Corrección del verificador o de la suite de pruebas.
3. Reversión: la versión defectuosa se desactiva, una versión histórica anterior válida pasa a oficial activo si existe. La versión defectuosa **no se borra**; se marca con flag `withdrawn` y causa.
4. Re-ejecución del pipeline con la suite corregida.

### Desactivación de un circuito

Es excepcional que un circuito entero se desactive. Procede solo si:

1. Su nombre canónico cambia (renombramiento de familia). Casos especificados con migración trazada.
2. Su semántica se considera obsoleta (rarísimo en AONIX, dado su mundo formal estable).
3. Una decisión documentada con auditoría completa.

En ningún caso esto borra historial.

## Política sobre versiones experimentales

Las versiones experimentales se guardan con:

- `experimental_status: failed_verification | not_promoted | regression | conflict`
- `cause: detail`
- `episode_origin: aonclg_id`
- `compared_against: hash_canonical?` — qué oficial activo perdió la comparación

Las experimentales **no compiten** como oficiales activas. Sirven para análisis, aprendizaje y auditoría.

## Política sobre versiones importadas

Si un humano (o sistema externo) importa un `.aoncir` desde fuera de un episodio:

1. Pasa el parser estricto.
2. Pasa el verificador con la suite de la tarea correspondiente.
3. Pasa el evaluador y la comparación con el oficial activo si existe.
4. Si gana, se promueve normalmente.

La importación queda registrada con `imported_from`, `imported_by`, `imported_at`. El `.aonclg` puede ser null (no hay trayectoria de aprendizaje), o puede sintetizarse uno mínimo con la causa de importación.

## Política sobre versiones generadas por búsqueda exhaustiva o solvers

AONIX puede usar agentes baseline como búsqueda exhaustiva o SAT solvers para producir circuitos. Estas versiones:

- Pasan exactamente las mismas puertas que cualquier otra.
- Tienen `agent: search_exhaustive_v1` o equivalente.
- Su `.aonclg` puede ser mínimo (la "trayectoria" de un solver no necesita capturarse en detalle).

Son ciudadanas de primera clase del versionado.

## Hashing y deduplicación

El `hash_canonical` se calcula sobre la topología abstracta del grafo (ver [03](03-format-aoncir.md)). Dos versiones con el mismo hash son la **misma versión estructural** y no se duplican en memoria canónica.

Si dos episodios distintos producen el mismo hash:

- Existe **una sola entrada canónica** (la primera que se registró).
- Sus `.aonclg` son distintos y referencian la misma entrada por hash.
- Las dos trayectorias se conservan; el destino estructural es uno solo.

## Coherencia entre memoria canónica e histórica

Garantías cruzadas:

1. **Toda versión histórica fue alguna vez oficial activa**, salvo importaciones explícitas marcadas con `direct_to_history: true` (raro, solo modo análisis).
2. **Toda versión oficial activa tiene predecesor en histórica o `predecessor: null`** (primera de su línea).
3. **La cadena `predecessor`/`successor` no tiene ciclos**.
4. **El hash canónico es único** dentro del par (memoria canónica ∪ memoria histórica) para un mismo `(name, parameters)`.
5. **Cualquier versión histórica puede consultarse** y simularse en cualquier momento.

## Re-verificación periódica

AONIX puede ejecutar **re-verificación periódica** sobre el oficial activo (y opcionalmente sobre el histórico) cuando:

- La suite de pruebas se actualiza con casos nuevos.
- El modelo de referencia cambia.
- El simulador o verificador se actualiza.

Si una versión oficial activa **falla** la re-verificación con suite ampliada, se marca como **degradada** y se inicia revisión (ver V.5).

## Política sobre versionado del propio AONIX

Cuando AONIX cambia (nueva versión del verificador, del evaluador, del optimizador, del parser):

- Las versiones canónicas e históricas **no se invalidan automáticamente**.
- La re-verificación es opcional, programable, y se registra en auditoría.
- Si un cambio en AONIX hace que una versión que pasaba ahora falle, el procedimiento es V.5 (degradación + revisión).

## Lo que la política **garantiza**

- Una sola verdad técnica vigente por circuito y tamaño.
- Historia completa para auditoría, comparación, evolución y entrenamiento.
- Transiciones atómicas y trazadas.
- Concurrencia sin pérdida.
- Ningún borrado silencioso.
- Reproducibilidad bit a bit de cualquier versión registrada.
