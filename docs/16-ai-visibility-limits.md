# 16 — Límites de visibilidad para la IA

> **Documento normativo.** Define **qué puede ver y qué no puede ver** un agente (humano, búsqueda o IA) durante un episodio. Sin estos límites, la memoria se convertiría en un catálogo de atajos, y la IA aprendería a buscar respuestas en lugar de construir lógica desde AND/OR/NOT. Este documento es la regla operativa que impide ese atajo.

## Principio

> Información es contexto, no atajo.

Un agente tiene derecho a información suficiente para razonar sobre la tarea (qué meta, qué entradas, qué salidas, qué acciones legales). Un agente **no tiene derecho** a ver la solución oficial activa de la tarea que está resolviendo, ni las trayectorias completas de otros agentes que la resolvieron, salvo en **modos de estudio explícitos** declarados por el currículo.

La separación entre **contexto** y **atajo** se materializa en una lista enumerable: `AgentVisibilitySet`.

## Conjunto de visibilidad por defecto durante un episodio

Salvo configuración explícita de la tarea, el agente ve:

### Visible siempre

1. **Metadatos de la tarea:** `id`, `name`, `version`, `level`, `family`, `tags`.
2. **Dominio:** `inputs`, `outputs`, `semantic_groups` con sus etiquetas.
3. **Especificación funcional:** la `spec` se entrega según política del nivel:
   - Niveles 0–4: tabla de verdad **completa** visible.
   - Niveles 5–10: tabla de verdad **completa o resumida**, según tarea; modelos de referencia accesibles como caja negra (entrada → salida).
   - Niveles 11+: spec temporal visible como caja negra (puede consultarse con vectores y secuencias).
4. **Acciones permitidas y prohibidas:** `allowed_actions`, `forbidden_actions`. El agente recibe la lista enumerable.
5. **Estado del episodio actual:**
   - Circuito parcial construido por el propio agente.
   - Señales actualmente disponibles.
   - Compuertas creadas hasta el momento.
   - Acciones legales **en este punto** del episodio (calculadas por el validador).
6. **Retroalimentación del validador** sobre cada acción propia (legal/ilegal + causa).
7. **Resultados del simulador** sobre entradas propuestas por el propio agente (acción `test_specific_input`).
8. **Métricas parciales del evaluador** sobre el circuito parcial actual (acción `request_evaluation`).
9. **Recompensa parcial acumulada** del episodio.
10. **Explicaciones del traductor humano** cuando el agente las solicita (acción `request_explanation`).
11. **Reglas absolutas** del sistema (R1, R2) — recordadas en cada estado para evitar deriva.

### Visible bajo modo explícito (currículo lo habilita)

12. **Versiones históricas de circuitos previamente verificados** — pero solo en **modo estudio**, no durante un episodio activo cuya tarea coincida.
13. **Trayectorias de otros agentes** sobre tareas distintas — para análisis comparativo, no como modelo a copiar.
14. **Catálogo de transformaciones** del optimizador — para aprender a sugerir, no para invocar primitivas nuevas.
15. **Memoria de fallos** del propio agente — qué patrones no funcionan para él.

### Nunca visible durante un episodio activo

16. **El circuito oficial activo de la tarea que el agente está resolviendo**, salvo modo estudio que cierra el episodio actual y abre uno separado de estudio.
17. **El `.aonclg` de otros agentes que ya resolvieron la misma tarea**, salvo modo análisis post-resolución.
18. **La semilla** de pruebas aleatorias, salvo si la política de la tarea la expone (para reproducibilidad). El conjunto de casos aleatorios concretos se entrega solo como resultado, no anticipadamente.
19. **Casos límite no aún ejercitados** del catálogo (para preservar valor de prueba sorpresa). Una vez ejercitados, se reportan; antes, no.
20. **Métricas del oficial activo** de la misma tarea — el agente sabe que existe, sabe que su meta puede ser superarlo, pero **no recibe sus números** salvo modo comparativo post-resolución.
21. **Solución de subtareas previas** que el agente no haya resuelto. (Si ya las resolvió, sus propios `.aoncir` son visibles como conocimiento personal.)
22. **Memoria de aprendizaje agregada de otros agentes**, salvo agregados estadísticos anonimizados (ej. "esta tarea tiene tasa de éxito 60%" sin trayectoria concreta).
23. **Información sobre futuros niveles**, salvo el siguiente nivel desbloqueable cuando esté cerca del avance.

## Lo que el agente puede inferir, pero AONIX no entrega

Hay información que el agente **puede deducir** legítimamente pero que AONIX **no entrega como dato directo**:

- Que XOR es expresable con AND/OR/NOT. El agente puede descubrir esa equivalencia construyéndola; AONIX no entrega "una receta de XOR".
- Que un full adder puede construirse con dos half adders. Si la composición es válida, el agente puede construirla; AONIX no entrega el patrón como blueprint.
- Que ciertas tareas se benefician de factorización. El agente puede aprender la técnica; AONIX no entrega "factoriza aquí".

La línea es clara: AONIX entrega **el mundo y la meta**, no el camino concreto.

## Modos de estudio (excepciones formales)

El currículo puede activar **modos de estudio** donde el agente ve más información, pero con condiciones:

- **Modo "review-after-solve":** tras resolver una tarea, el agente puede ver el `.aoncir` oficial activo y compararlo con su propia solución. El episodio activo ya cerró. No hay re-promoción basada en este modo.
- **Modo "study-historical":** el agente puede examinar versiones históricas de un circuito ya resuelto en niveles previos. Útil para entender evolución. No habilita copia directa en episodios activos.
- **Modo "guided-onboarding":** en niveles iniciales (0–1), AONIX puede mostrar fragmentos de soluciones modelo para ilustrar el mundo formal. **Estos fragmentos son ejemplos pedagógicos, no soluciones canónicas reutilizables.**
- **Modo "post-mortem":** tras un fallo, el agente puede recibir información adicional sobre el caso fallido y, opcionalmente, un hint estructural ("la salida `carry` no se activa cuando ambas entradas son 1"). Sin entrega de la solución.

Cada modo se declara explícitamente en la tarea o en el episodio. Sin declaración, se aplica el régimen por defecto.

## Visibilidad sobre el propio progreso

El agente siempre puede ver:

- Su propia tasa de éxito por nivel.
- Sus propias trayectorias pasadas (`.aonclg` propios).
- Su progreso curricular individual.
- Los criterios de avance de su nivel actual.

Esto es razonable: el agente conoce su propio historial.

## Visibilidad sobre el sistema AONIX

El agente puede consultar:

- Las reglas absolutas (R1, R2).
- La estructura de la tarea actual.
- El catálogo de tipos de acción.
- El esquema de retroalimentación.
- Las versiones de AONIX, suites y modelos de referencia.

Nada de esto es atajo. Es contrato entre agente y mundo.

## Visibilidad asimétrica entre agentes

Distintos agentes corriendo episodios paralelos:

- **No ven** las acciones del otro durante el episodio.
- **No comparten** progreso curricular (cada uno tiene el suyo).
- **No ven** la trayectoria intermedia del otro.

Al cierre de cada episodio:

- Las soluciones aceptadas entran en memoria canónica si mejoran; ambas trayectorias se archivan.
- Si uno promueve y otro no, el segundo se entera al iniciar el siguiente episodio.
- Las memorias agregadas (tasas, patrones) pueden compartirse anonimizadas.

## Visibilidad sobre la memoria

La separación se trata en detalle en [18 — Memoria operativa vs no operativa](18-operational-vs-non-operational-memory.md). Resumen rápido:

| Memoria | Visible al agente durante episodio | Usable como atajo |
|---------|------------------------------------|-------------------|
| Canónica (de la tarea actual) | **No** (por defecto) | Nunca |
| Canónica (de otras tareas que el agente resolvió) | Sí (sus propios `.aoncir`) | No, son entidades canónicas, no compuertas |
| Histórica | Solo modo estudio | No |
| Aprendizaje (propia) | Sí | No |
| Aprendizaje (ajena) | Solo post-resolución | No |
| Experimental | Solo agregados | No |
| Pruebas | Solo descripción, no casos específicos | No |
| Visual | Solo del circuito actual | No |
| Curricular (propia) | Sí | No |
| Trayectorias (propias) | Sí | No |
| Trayectorias (ajenas) | Solo modo estudio | No |
| Fallos (propios) | Sí | No |
| Optimización | Solo agregado / catálogo | No |

## Implementación: la cara visible es enumerable

El traductor para IA (capa 12, ver [02](02-architecture.md)) **construye y entrega** explícitamente el conjunto visible en cada paso. No hay "información oculta accesible por error": si AONIX no la añade al payload del estado, la IA no la recibe.

Esquema simplificado:

```
AgentState {
    task_view:           TaskView        # filtrada por visibility set
    episode_view:        EpisodeView     # estado actual
    legal_actions:       [Action]        # enumerable
    feedback_recent:     [Feedback]      # del propio agente
    rewards_running:     RewardSummary
    rules_reminder:      AbsoluteRules   # R1, R2
    optional_views:      OptionalViews   # solo si la tarea lo permite
}
```

## Pruebas de filtración

AONIX incluye **tests de filtración**: garantizan que ningún canal lateral (logs, mensajes de error, métricas debug, traducciones humanas mal pensadas) revele información que no esté en `AgentVisibilitySet`.

Ejemplos de tests:

- **Test de spec leakage:** verificar que el mensaje de error de un L1 no incluye la solución, solo la diferencia caso a caso.
- **Test de canonical leakage:** verificar que el evaluador, al rechazar una promoción, no entrega las métricas del oficial activo en bruto, solo el delta cualitativo.
- **Test de historical leakage:** verificar que las explicaciones del traductor humano no citan literalmente el `.aoncir` oficial activo cuando se está resolviendo esa misma tarea.

Si un test de filtración falla en CI, el build falla.

## Visibilidad y reward hacking

La separación contexto/atajo cierra una vía de reward hacking: un agente que aprende a "preguntar" información que no debería ver para reconstruir la solución. AONIX cierra esa vía por construcción al hacer la visibilidad **enumerable** y **filtrada en origen**, no por confianza.

## Lo que la visibilidad **nunca** debe permitir

- Que un agente lea el `.aoncir` oficial activo de la tarea que está resolviendo.
- Que un agente obtenga las primitivas derivadas como compuertas listas (no existen).
- Que un agente extraiga la spec completa cuando la política de la tarea sea caja negra.
- Que un agente reconstruya la solución oficial activa a partir de respuestas del simulador (esto es un caso aceptable de "aprendizaje", pero la spec no debe filtrarse en una sola consulta).
- Que la visualización muestre el oficial activo durante el episodio que lo intenta superar.
- Que dos agentes paralelos colisionen en información lateral.

## Auditoría de visibilidad

Cada episodio cerrado deja en el `.aonclg`:

- El `AgentVisibilitySet` efectivo aplicado.
- Las consultas realizadas por el agente.
- Las respuestas servidas (con filtros aplicados).

Esto permite auditar a posteriori si una política de visibilidad permitió un atajo no intencionado.
