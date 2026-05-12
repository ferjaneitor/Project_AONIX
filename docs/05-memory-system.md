# 05 — Sistema de memorias

## Principio general

AONIX **no tiene una memoria, tiene varias**. Cada memoria cumple un rol distinto, tiene políticas de escritura propias y reglas de uso separadas. Mezclarlas es un anti-patrón.

**Regla absoluta de memoria:** ninguna memoria puede usarse como atajo que viole las reglas absolutas (R1 y R2). La memoria conserva información, pero la construcción final de un circuito siempre debe permanecer expresada exclusivamente con AND, OR y NOT.

## Las 10 memorias

| # | Nombre | Qué guarda | Mutable | Atajo permitido |
|---|--------|-----------|---------|-----------------|
| 1 | Memoria canónica | `.aoncir` oficial activo por circuito y tamaño | Por reemplazo atómico | No |
| 2 | Memoria histórica de versiones | Versiones anteriores verificadas | No (append-only) | No |
| 3 | Memoria de aprendizaje | `.aonclg` y datos para entrenamiento | Append-only | No |
| 4 | Memoria experimental | Intentos, variantes, fracasos, benchmarks | Append-only | No |
| 5 | Memoria de pruebas | Suites de validación por nivel | Versionada | No |
| 6 | Memoria visual | Layouts 2D, posiciones, agrupaciones, conos, buses | Mutable con auditoría | No |
| 7 | Memoria curricular | Progreso, niveles, tareas dominadas/pendientes | Mutable con auditoría | No |
| 8 | Memoria de trayectorias | Paso a paso de construcción | Append-only | No |
| 9 | Memoria de fallos | Patrones inválidos, errores recurrentes | Append-only | No |
| 10 | Memoria de optimización | Transformaciones exitosas, mejoras, comparaciones | Append-only | No |

## 1. Memoria canónica

**Propósito:** servir la verdad técnica vigente.

**Contenido:** un único `.aoncir` oficial activo por circuito y tamaño.

**Acceso:**
- **Lectura:** simulador, verificador, visualizador, traductor, evaluador, coordinador.
- **Escritura:** únicamente el coordinador, previa decisión del verificador y del evaluador.

**Política de reemplazo:** una versión nueva reemplaza la oficial activa si y solo si:

1. Pasa el verificador en el nivel correspondiente (correctitud).
2. Mejora estrictamente bajo el ranking del evaluador (criterio configurable, por defecto: conteo de compuertas, profundidad, reutilización en este orden lexicográfico).
3. El reemplazo se registra en memoria histórica.
4. La operación es atómica: o se completa íntegro (incluye actualización del hash predecessor, archivo de la versión anterior, refresco de índices) o se aborta.

**Identificación:** por nombre canónico + parámetros (típicamente `width`). Ejemplos:

- `one_bit_full_adder.aoncir`
- `four_bit_full_adder.aoncir`
- `thirty_two_bit_full_adder.aoncir`

## 2. Memoria histórica de versiones

**Propósito:** preservar la evolución de cada circuito.

**Contenido:** todas las versiones verificadas que alguna vez fueron oficiales activas o que pasaron el verificador y se conservaron como referencia.

**Política:** append-only. **Nunca se borra historia.**

**Usos:**

- Comparar versiones nuevas contra anteriores.
- Medir mejora de compuertas, profundidad, reutilización.
- Detectar regresiones (una "mejora" que rompe pruebas que la anterior superaba se rechaza).
- Analizar trayectorias de optimización.
- Explicar cómo evolucionó un circuito a humanos.
- Entrenar modelos con ejemplos de mejora (par antes/después).
- Auditar decisiones del sistema.
- Recuperar una versión anterior si una optimización posterior resulta peor bajo otro criterio (el evaluador puede tener ranking multi-objetivo configurable).

**Indexación:** por nombre canónico + parámetros + hash canónico + timestamp.

## 3. Memoria de aprendizaje

**Propósito:** alimentar el entrenamiento futuro de IA y registrar el contexto formativo.

**Contenido:** archivos `.aonclg` (ver [04 — Formato `.aonclg`](04-format-aonclg.md)), organizados por tarea, nivel y agente.

**Política:** append-only. Cada episodio cerrado produce un `.aonclg` inmutable.

**Restricción central:** la memoria de aprendizaje **no entrega atajos**. Un agente que aprende puede recibir trayectorias, pares (estado, acción), recompensas, pero nunca recibe "el circuito final como compuerta importable". La memoria de aprendizaje informa, no sustituye.

## 4. Memoria experimental

**Propósito:** guardar lo que se probó y no se promovió.

**Contenido:**

- Intentos de circuitos que no pasaron pruebas.
- Variantes parcialmente correctas.
- Benchmarks de rendimiento (tiempo de simulación, validación, optimización).
- Comparaciones entre estrategias.
- Branches de exploración descartados.

**Política:** append-only con etiquetas de motivo de descarte (failed_verification, suboptimal, regression, etc.).

**Uso:** análisis ex-post, debugging del propio AONIX, identificación de patrones que llevan a fracaso.

## 5. Memoria de pruebas

**Propósito:** conservar las suites de validación.

**Contenido:**

- Suite exhaustiva por nivel pequeño.
- Suite aleatoria con semilla fija por nivel mediano.
- Casos límite catalogados.
- Suite de regresión por circuito (acumulada con el tiempo: cada caso fallido detectado se añade).
- Suite por propiedades.
- Modelos de referencia.

**Política:** versionada. Una suite puede crecer (añadir casos) pero los casos existentes no se modifican sin justificación auditable. Eliminar un caso requiere acuerdo explícito y queda registrado.

**Identificación:** `suite_id` con versión.

## 6. Memoria visual

**Propósito:** persistir layouts 2D y agrupaciones visuales.

**Contenido:**

- Posiciones 2D de nodos.
- Agrupaciones (regiones nombradas, bloques colapsables).
- Rutas críticas precomputadas.
- Conos lógicos precomputados por salida.
- Buses agrupados por etiqueta semántica.
- Estilos visuales asociados a tags.

**Política:** mutable con auditoría. Un layout puede recomputarse (auto-layout), pero los layouts curados manualmente se preservan.

**Restricción:** la memoria visual **no afecta la verdad técnica**. Cambiar el layout no cambia el `.aoncir`. El hash canónico no depende del layout.

## 7. Memoria curricular

**Propósito:** registrar el progreso a través de niveles.

**Contenido:**

- Niveles completados por agente.
- Tareas dominadas vs pendientes.
- Métricas agregadas por nivel (tasa de éxito, tiempo medio, intentos medios).
- Condiciones de avance evaluadas.
- Desbloqueos.

**Política:** mutable con auditoría. El avance es trazable.

**Restricción:** un agente solo avanza por demostración de dominio, nunca por tiempo, nunca por intervención manual no auditable.

## 8. Memoria de trayectorias

**Propósito:** registrar el camino exacto de construcción de cada circuito relevante.

**Contenido:** secuencia de acciones, validaciones, simulaciones intermedias, retroalimentaciones, hasta el cierre del episodio.

**Política:** append-only. Inmutable tras cierre.

**Diferencia con memoria de aprendizaje:** la trayectoria es la materia prima; la memoria de aprendizaje (`.aonclg`) la envuelve con etiquetas, recompensas y contexto curricular.

## 9. Memoria de fallos

**Propósito:** acumular conocimiento sobre lo que **no** funciona.

**Contenido:**

- Patrones de circuito inválidos (con ciclo, con señal muerta crítica, con conexión a inexistente).
- Errores recurrentes detectados por el validador.
- Caminos de construcción que tienden a no converger.
- Tareas con tasa de fracaso anormalmente alta.

**Política:** append-only con categorización.

**Uso:** retroalimentación al traductor humano, ajuste del currículo, detección de fallos sistémicos del propio AONIX.

## 10. Memoria de optimización

**Propósito:** registrar transformaciones que mejoran circuitos.

**Contenido:**

- Pares (antes, después) con tipo de transformación aplicada (eliminación de redundancia, factorización, De Morgan, absorción, idempotencia, etc.).
- Métricas delta (reducción de compuertas, de profundidad, etc.).
- Casos en los que una optimización falló (regresión detectada al re-verificar).

**Política:** append-only.

**Uso:** alimentar al optimizador estructural, guiar la búsqueda, entrenar modelos que aprendan a optimizar.

## Diagrama de flujos de memoria durante un episodio

```
[Agente] → [Validador] → [Simulador] → [Verificador] → [Evaluador]
                                                            │
                                                            ▼
                                                   ¿reemplaza oficial?
                                                            │
                                              ┌─────────────┴─────────────┐
                                              ▼                           ▼
                                          sí, mejor                  no, descartar
                                              │                           │
            ┌─────────────────────────────────┘                           │
            │                                                             │
            ▼                                                             ▼
    Memoria canónica ←── reemplazo atómico         Memoria experimental ← intento fallido o subóptimo
            │
            ▼
    Memoria histórica ←── archivar versión anterior
            │
            ▼
    Memoria de optimización ←── delta antes/después
            │
            ▼
    Memoria de aprendizaje ←── nuevo .aonclg con trayectoria + recompensa
            │
            ▼
    Memoria curricular ←── actualizar progreso si aplica
```

Memoria visual y memoria de pruebas se consultan/actualizan en paralelo según hagan falta. Memoria de trayectorias y memoria de fallos se alimentan paso a paso durante el episodio.

## Reglas inviolables de memoria

1. **Una sola versión oficial activa.** Por circuito + tamaño. Sin excepciones.
2. **Append-only para historia.** Lo que se aprendió o se intentó no se borra.
3. **Sin atajo.** Ninguna memoria entrega compuertas derivadas como primitivas importables.
4. **Reemplazo atómico.** Promover una versión nueva a oficial activa es transaccional.
5. **Trazabilidad completa.** Toda escritura registra: quién (agente / módulo), cuándo, por qué (decisión del verificador y evaluador), qué (delta y hashes).
6. **Separación de roles.** La visualización no escribe en memoria canónica. La traducción para IA no escribe en memoria histórica. El validador no escribe en memoria de aprendizaje. Cada memoria tiene escritores explícitos.

## Almacenamiento físico (decisión pendiente)

Alternativas razonables:

- **Sistema de archivos plano** con índices JSON/TOML — simple, transparente, auditable.
- **Embedded DB** (SQLite, sled, redb) — más rápido para consultas e índices secundarios.
- **Híbrido** — `.aoncir` y `.aonclg` como archivos planos (la verdad permanece auditable a ojo), índices y metadatos en DB embebida.

**Recomendación inicial:** **híbrido**. La verdad técnica vive en archivos; las consultas eficientes pasan por DB embebida que indexa los archivos. La decisión final corresponde al usuario.
