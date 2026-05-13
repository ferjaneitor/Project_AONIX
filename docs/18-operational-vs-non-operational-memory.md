# 18 — Memoria operativa vs memoria no operativa

> **Documento normativo.** Refina la taxonomía de [05 — Sistema de memorias](05-memory-system.md) introduciendo una distinción crítica: qué memorias se pueden usar **como atajo operativo durante un episodio activo**, y cuáles **no pueden serlo nunca, aunque existan y sean consultables**.

## Por qué importa esta distinción

AONIX guarda muchísima información: oficiales activos, históricos, aprendizaje, fallos, optimizaciones, layouts. Toda esta información tiene **valor de auditoría y aprendizaje**, pero solo una **fracción** puede consumirse como insumo directo dentro de un episodio activo.

La distinción operativa / no operativa cierra una puerta de atajo:

- **Memoria operativa** = información que un agente puede leer y usar para tomar decisiones en su episodio activo.
- **Memoria no operativa** = información que existe, se conserva, se audita, sirve para entrenamiento y análisis posterior, pero **no se le entrega al agente** durante su episodio activo como insumo.

Sin esta separación, "tener historial" se convertiría en "tener biblioteca de soluciones", y AONIX dejaría de cumplir su propósito pedagógico.

## Clasificación

Aplicada a las 10 memorias de AONIX:

| # | Memoria | Operativa para agente activo | No operativa |
|---|---------|------------------------------|--------------|
| 1 | Canónica | **Parcial**: solo si NO está resolviendo esa tarea | **Completa**: cuando coincide con tarea activa |
| 2 | Histórica | **No** durante episodio activo de esa tarea | **Sí** siempre (excepto modo estudio explícito) |
| 3 | Aprendizaje (`.aonclg`) | **Solo propios** del agente activo | Ajenos durante episodio activo |
| 4 | Experimental | **No** como referencia directa | **Sí** siempre |
| 5 | Pruebas | **No** sus casos concretos antes de evaluarse | Casos individuales no revelados |
| 6 | Visual | **Sí** para circuito propio del episodio actual | Layouts de oficiales activos de la tarea actual |
| 7 | Curricular | **Sí** la del propio agente | Ajena |
| 8 | Trayectorias | **Solo propias** del agente | Ajenas durante episodio activo |
| 9 | Fallos | **Solo propios** del agente | Ajenos |
| 10 | Optimización | **Catálogo de transformaciones** sí; pares concretos solo del oficial activo de tareas previas resueltas | Pares de la tarea activa |

## Definiciones formales

### Memoria operativa

Una memoria es **operativa para un agente A en un episodio E** si y solo si:

1. AONIX entrega su contenido al agente A como parte del `AgentState` durante E, **o**
2. AONIX permite consulta explícita por A durante E vía acciones declaradas legales.

Si una memoria es operativa, su lectura por A es legítima y registrada.

### Memoria no operativa

Una memoria es **no operativa para A en E** si AONIX no la incluye en `AgentState` ni la expone vía acción legal durante E. Su contenido sigue existiendo en disco, se conserva con todas las garantías de durabilidad, se audita, se usa para entrenamiento offline, pero **no influye en E**.

## Reglas por memoria

### 1. Memoria canónica

- **Operativa** cuando el agente consulta el `.aoncir` de **otra tarea** que ya resolvió, como subcomponente conceptual de su tarea actual. Cuidado: lo que el agente lee es la **entidad canónica** (composición jerárquica), pero al producir su propio `.aoncir` debe expandir a primitivas. La consulta canónica de otra tarea **no convierte ese circuito en una primitiva invocable**.
- **No operativa** cuando el agente está resolviendo precisamente esa tarea. Se le oculta. Ver [16 — Visibilidad](16-ai-visibility-limits.md).

### 2. Memoria histórica

- **Operativa** solo en **modo estudio** que el currículo habilita explícitamente fuera de un episodio activo de la misma tarea.
- **No operativa** durante un episodio activo cuya tarea coincida con el circuito histórico.

### 3. Memoria de aprendizaje (`.aonclg`)

- **Operativa**: los propios `.aonclg` del agente. El agente recuerda sus propios episodios.
- **No operativa**: `.aonclg` de otros agentes durante el episodio activo. Pueden ser operativos en post-mortem o modo análisis comparativo, nunca en línea con un episodio sobre la misma tarea.

### 4. Memoria experimental

- **No operativa por defecto** durante episodios activos. Los descartes acumulados podrían ser atajo si se entregan crudos.
- **Operativa solo de forma agregada y anonimizada** (estadísticas tipo "el 78% de los intentos en esta familia falla por señal muerta") sin entregar los `.aoncir` experimentales en sí.

### 5. Memoria de pruebas

- **Operativa**: la **descripción** de las suites (qué pruebas se aplican, qué pasos, qué tipos de casos) puede leerse para que el agente sepa qué se le va a exigir.
- **No operativa**: los **casos concretos** que aún no se han ejecutado. Se preserva el valor sorpresa.
- **Operativa post-evaluación**: cuando una suite ya se evaluó sobre el circuito, los casos fallidos concretos pasan al agente como retroalimentación.

### 6. Memoria visual

- **Operativa**: layouts y agrupaciones del **circuito en construcción** del propio agente. Estos son del agente.
- **No operativa**: layouts curados del oficial activo de la tarea actual (durante el episodio).
- **Operativa post-resolución**: layouts del oficial activo cuando el episodio ya cerró y se entra en modo revisión.

### 7. Memoria curricular

- **Operativa**: el progreso del propio agente.
- **No operativa**: el progreso ajeno (salvo agregados anónimos).

### 8. Memoria de trayectorias

- **Operativa**: trayectorias del propio agente, especialmente la del episodio actual.
- **No operativa**: trayectorias ajenas durante el episodio activo.

### 9. Memoria de fallos

- **Operativa**: fallos del propio agente. Aprende de sus propios errores.
- **No operativa**: fallos ajenos durante el episodio activo (salvo patrones agregados anónimos como "esta tarea tiene tasa de fallo X en señal Y").

### 10. Memoria de optimización

- **Operativa**: el **catálogo de transformaciones** disponibles (cuáles existen, qué hacen). El agente puede saber que existe "factorización booleana" como concepto.
- **No operativa**: los pares concretos (antes, después) del oficial activo de la tarea activa. Esos serían atajo.
- **Operativa para tareas resueltas**: los pares de tareas que el agente ya resolvió pueden mostrarse en modo revisión.

## Operatividad asimétrica entre agentes

| Memoria | Agente A activo | Agente B activo |
|---------|----------------|-----------------|
| Canónica de tarea T (que A resuelve) | No op. para A | Op. para B si B no resuelve T |
| `.aonclg` de A | Op. para A | No op. para B |
| `.aonclg` de B | No op. para A | Op. para B |
| Curricular de A | Op. para A | Agregado para B |
| Histórica | Modo estudio | Modo estudio |

La operatividad es **por agente** y **por episodio**, no global.

## Operatividad temporal

Algunas memorias cambian de operatividad según el momento:

- **Antes del episodio**: el catálogo de pruebas es operativo (descripción); los casos concretos no.
- **Durante el episodio**: los casos fallidos detectados se vuelven operativos como retroalimentación.
- **Después del episodio**: el `.aoncir` oficial activo de la tarea se vuelve operativo en modo revisión.

## Operatividad por modo

Los modos del currículo (ver [16](16-ai-visibility-limits.md) § Modos de estudio) modifican qué es operativo:

- **review-after-solve**: tras cerrar episodio, oficial activo y comparación se vuelven operativos.
- **study-historical**: memoria histórica se vuelve operativa para análisis.
- **guided-onboarding**: fragmentos de soluciones modelo son operativos (solo niveles iniciales).
- **post-mortem**: tras fallo, más información se vuelve operativa.

En cada caso, **se declara explícitamente** qué pasa de no-operativo a operativo. Sin declaración, rige el régimen por defecto.

## Por qué la separación no es solo "ocultar"

Una memoria no operativa **se conserva con todas las garantías**:

- Durabilidad (no se borra).
- Auditoría (se puede inspeccionar externamente).
- Reproducibilidad (se puede re-ejecutar).
- Análisis offline (post-mortem, agregados, métricas globales).
- Entrenamiento de modelos (datasets supervisados, RL, distillation).

Lo que **no** ocurre es que se entregue al agente **dentro** de su episodio activo como input para sus decisiones. La diferencia es la **temporalidad operativa**, no la existencia.

## Mecanismos de cumplimiento

1. **Capa de traducción para IA (capa 12)**: construye el `AgentState` filtrando por operatividad. Si una información es no operativa, **no se incluye**.
2. **Validador de acciones**: rechaza consultas que intenten acceder a memoria no operativa.
3. **Tests de filtración** (ver [16](16-ai-visibility-limits.md)): garantizan que canales laterales no filtren información no operativa.
4. **Auditoría por episodio**: el `.aonclg` registra qué memorias estuvieron operativas durante el episodio. Cualquier discrepancia entre política declarada y operatividad efectiva se detecta a posteriori.

## Lista de no-negociables

- **La memoria canónica de la tarea activa nunca es operativa** para el agente que está resolviendo esa tarea, salvo modo de estudio post-episodio.
- **Las trayectorias ajenas nunca son operativas** durante episodios activos.
- **Los casos concretos de pruebas no ejecutadas nunca son operativos**, salvo que la política de la tarea los declare expuestos por reproducibilidad.
- **La memoria experimental nunca es operativa cruda**, solo agregada.
- **El catálogo de transformaciones del optimizador siempre es operativo** (es conocimiento del mundo formal, no atajo); los pares concretos pueden ser o no operativos según la tarea.

## Operatividad y reward hacking

Una IA podría intentar:

- Construir consultas que reconstruyan información no operativa.
- Aprovechar ambigüedades en la frontera entre operativo y no operativo.
- Solicitar modos de estudio donde no corresponden.

AONIX mitiga estas vías con:

- **Enumeración explícita** del `AgentState`: si algo no está en el payload, no se reconstruye.
- **Validador estricto** de acciones de consulta: solo modos declarados se activan.
- **Auditoría continua** por episodio: patrones de explotación se detectan offline.

## Visualización de la línea operativa

```
   episodio activo de tarea T por agente A
   ───────────────────────────────────────────────────────────────────►

   t0          t1          t2          t3          t_close      t+epsilon
   │           │           │           │           │             │
   inicio   acción 1    acción 2   stop_const.   verificación   modo review
   │           │           │           │           │             │
   │           │           │           │           │             │
   ─────────┼─────────┼─────────┼─────────┼─────────────┼─────────►
   │           │           │           │           │             │
   estado     estado     estado     estado     veredictos     oficial activo
   inicial    parcial    parcial    final      finales        AHORA op. (revisión)
                                              (resultado)
   │                                                          │
   ▼                                                          ▼
   visibilidad mínima                                       visibilidad ampliada
   (no operativo: canónica T, .aonclg ajeno, casos          (operativo: comparación,
   límite no ejercitados...)                                histórico relacionado...)
```

## Resumen ejecutivo

- AONIX guarda casi todo. AONIX **entrega muy poco** durante un episodio activo.
- La diferencia entre **guardar** y **entregar al agente activo** es la frontera operativa.
- El propósito es preservar valor pedagógico: el agente construye desde AND/OR/NOT con contexto pero sin atajos.
- La línea es **enumerable, auditable, configurable por modos**, y se hace cumplir por construcción.
