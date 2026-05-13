# 14 — Reglas de rechazo

> **Documento normativo.** Catálogo exhaustivo de causas por las que AONIX **rechaza** una acción, un circuito o un `.aoncir`. Complementa a [13 — Reglas de aceptación](13-circuit-acceptance.md): toda condición que viole una regla de aceptación es causa de rechazo aquí, pero este documento añade el cómo, dónde y qué retroalimentación se emite.

## Principio

> Rechazo temprano > rechazo tardío. Rechazo informativo > rechazo silencioso.

Todo rechazo en AONIX cumple tres garantías:

1. **Determinista.** Misma acción + mismo estado ⇒ mismo veredicto.
2. **Trazable.** Toda causa de rechazo se registra con su categoría y localización exacta.
3. **Informativa.** El agente (humano o IA) recibe retroalimentación estructurada con la razón.

Un rechazo **no es** un error fatal del sistema. Es operación normal del mundo formal.

## Niveles de rechazo

| Nivel | Dónde se aplica | Reversible | Severidad |
|-------|----------------|------------|-----------|
| L0 — Acción rechazada | Validador de acciones | Sí (intenta otra) | Baja |
| L1 — Circuito rechazado por verificador | Verificador | Sí (intenta otra construcción) | Media |
| L2 — Circuito rechazado por evaluador (no promoción) | Evaluador | Sí (intenta otra versión) | Media |
| L3 — `.aoncir` rechazado por parser | Parser canónico | No durante carga | Alta |
| L4 — Promoción rechazada por coordinador | Coordinador | No (queda en experimental) | Alta |
| L5 — Tarea rechazada por cargador | Cargador de tareas | No (la tarea no se activa) | Crítica |

## L0 — Acción rechazada por el validador

El validador rechaza una acción **antes** de ejecutarla. El estado del circuito no cambia. El episodio continúa.

### Causas L0 (lista exhaustiva)

**Por violación de reglas absolutas:**

1. `create_gate_*` con tipo distinto de `AND`, `OR`, `NOT`.
2. Acciones del tipo `use_xor`, `use_xnor`, `use_nand`, `use_nor`, o cualquier variante que cree un nodo prohibido. **Rechazo categórico, penalización fuerte en recompensa.**
3. `import_gate` con `kind: derived`. **Rechazo categórico.**
4. Acción que produciría un nodo cuya expansión introduce un tipo prohibido.

**Por inconsistencia estructural:**

5. Compuerta `NOT` con aridad distinta de 1.
6. Compuerta `AND` u `OR` con aridad distinta de 2 (aridad estricta de Fase 1; ver [01 — Reglas absolutas](01-rules-absolute.md)).
7. Acción que referencia una señal indefinida.
8. Acción que duplica un identificador ya existente (señal, nodo).
9. Acción que crearía un ciclo no permitido en el nivel actual.
10. Asignación de salida del circuito a una señal inexistente.
11. Conexión con tipos incompatibles.
12. Acción que viola la representación 2D (coordenadas inválidas, posiciones imposibles).

**Por política de nivel/tarea:**

13. Acción no presente en `task.allowed_actions`.
14. Acción listada en `task.forbidden_actions`.
15. Acción que solo está habilitada en niveles superiores al actual.

**Por meta-políticas del sistema:**

16. Intento de saltarse el validador (`bypass_validator: true`). **Rechazo absoluto.**
17. Intento de declarar el circuito como correcto sin pasar por el verificador (`declare_correct`). **Rechazo absoluto.**
18. Intento de escribir directamente en memoria canónica sin promoción del coordinador. **Rechazo absoluto.**
19. Intento de modificar `.aonclg` ajeno o cerrado.
20. Acción tras `stop_construction` (el episodio está en cierre).

**Por límites del episodio:**

21. Acción tras `max_steps` agotados.
22. Acción tras `max_wall_time` agotado.

### Retroalimentación L0

Cada rechazo L0 emite:

```
ValidatorFeedback {
    action_id:       ActionId
    decision:        REJECTED
    cause_category:  ABSOLUTE_RULE | STRUCTURAL | TASK_POLICY |
                     META_POLICY | EPISODE_LIMIT
    cause_code:      enumerado (1..22)
    cause_message:   String              # legible
    relevant_state:  StateSnapshot       # qué del circuito impidió
    suggested_legal_actions: [Action]    # opcionalmente (modo enseñanza)
    reward_delta:    < 0                 # penalización
}
```

## L1 — Circuito rechazado por verificador

Al cierre del episodio, el verificador ejecuta las suites. Si **alguna** suite falla, el circuito se rechaza como solución.

### Causas L1

1. Cualquier vector de entrada produce salida distinta a la esperada (tabla de verdad).
2. Cualquier propiedad declarada se viola.
3. La comparación con `reference_model` falla.
4. Casos límite del catálogo de la tarea fallan (y la tarea los marca como `blocking`).
5. La tasa de éxito en pruebas aleatorias está por debajo de `success_criteria.minimum_pass_rate_random`.
6. Validación temporal falla (en tareas con `TemporalSpec`).
7. Validación por señal semántica falla (ej. salida etiquetada `carry` no se comporta como acarreo).
8. **Regresión:** el candidato falla una prueba que el oficial activo supera.

### Retroalimentación L1

```
VerifierFeedback {
    suite_id:             SuiteRef
    decision:             FAIL
    cause_category:       FUNCTIONAL | TEMPORAL | SEMANTIC | REGRESSION
    failing_cases:        [Case]
    first_failing_case:   Case             # cuál fue el primero
    expected_vs_produced: Diff
    regression_against:   CanonicalHash?   # si aplica
}
```

Un L1 **no es atajable**: ni la elegancia ni la optimización ni la recompensa compensan un fallo funcional.

## L2 — Circuito rechazado por evaluador (no promoción)

El evaluador no decide correctitud, pero sí decide **calidad relativa**. Un circuito correcto pero peor que el oficial activo no lo reemplaza.

### Causas L2

1. **Empate** según `ranking_order` ⇒ gana el incumbente.
2. **Peor** según `ranking_order` en al menos una dimensión sin compensar en otras dominantes.
3. **Umbrales mínimos** de `metric_thresholds` no alcanzados.
4. **Mejora marginal** por debajo del epsilon configurable de la tarea (anti-ruido).

### Retroalimentación L2

```
EvaluatorFeedback {
    decision:        NOT_PROMOTED
    cause_category:  NO_STRICT_IMPROVEMENT | THRESHOLD_BREACH | TIE
    metrics_candidate: {Metric -> Value}
    metrics_incumbent: {Metric -> Value}
    ranking_order:    [Metric]
    delta:            {Metric -> Delta}
}
```

L2 no es fracaso de la tarea (el circuito puede haberse aceptado como solución correcta), solo no reemplaza al oficial activo. Va a **memoria experimental** con causa registrada.

## L3 — `.aoncir` rechazado por parser

Carga de archivo, antes de cualquier validación dinámica. El parser es estricto: **no hay modo permisivo**.

### Causas L3

1. Sintaxis inválida del formato físico.
2. Campos obligatorios ausentes (`name`, `version`, `format_version`, `level`, ...).
3. `format_version` incompatible con el sistema.
4. Nodo de tipo distinto de AND, OR, NOT.
5. Aridad incorrecta para el tipo.
6. Señal referenciada antes de declararse.
7. Nombres duplicados.
8. Ciclos no permitidos.
9. Salidas no asignadas.
10. Hash canónico declarado ≠ hash computado.
11. Predecesor referenciado inexistente en memoria histórica (en modo estricto).
12. Layout con coordenadas no 2D o inválidas.
13. Etiqueta semántica no reconocida (modo estricto) o conflicto entre etiquetas (ej. señal marcada `clock` y `bus` simultáneamente sin convención válida).

### Retroalimentación L3

```
ParserError {
    file:            Path
    line:            N            # cuando aplique
    cause_code:      enumerado
    cause_message:   String
    recoverable:     false
}
```

Un `.aoncir` rechazado por parser **no entra en ninguna memoria**.

## L4 — Promoción rechazada por coordinador

El coordinador es el único que escribe en memoria canónica. Puede rechazar una promoción por:

### Causas L4

1. **Verificador no entrega `PASA`** sobre todas las suites de la tarea.
2. **Evaluador entrega L2** y la tarea no permite empate ni mejora marginal.
3. **Regresión post-optimización:** la versión optimizada falla pruebas que la pre-optimización pasaba.
4. **Conflicto de concurrencia:** otro episodio ya promovió una versión más reciente; el candidato actual debe re-evaluarse contra el nuevo incumbente.
5. **`task.canonical_promotion: block`:** la tarea no permite promoción (tareas de práctica).

### Retroalimentación L4

Se registra en memoria experimental y en el `.aonclg` del episodio. Sin pérdida: el circuito sigue siendo solución aceptada de la tarea, simplemente no se promueve.

## L5 — Tarea rechazada por cargador

El cargador de tareas verifica la conformidad de cada tarea antes de activarla.

### Causas L5

1. La tarea referencia `allowed_actions` que incluyen creación de nodo distinto a AND/OR/NOT.
2. La tarea modifica `forbidden_actions` para permitir compuertas prohibidas.
3. La tarea declara `expected_behavior` que sugiere usar primitivas no permitidas ("usa XOR…").
4. La tarea referencia suites de pruebas inexistentes o con hash desalineado.
5. La tarea referencia un `reference_model` inexistente.
6. La tarea declara `success_criteria` que omite el veredicto del verificador.
7. La tarea declara `bypass_validator: true`.
8. La tarea otorga al agente visibilidad sobre la solución oficial activa fuera de los modos de estudio permitidos por el currículo.
9. La tarea tiene esquema malformado.
10. La tarea tiene versión repetida sin incremento de `version`.

Una tarea rechazada en L5 **nunca se activa**. El catálogo la marca como inválida y el sistema requiere intervención humana para corregirla.

## Matriz: causa → módulo que rechaza → severidad

| Causa | Módulo | Severidad |
|-------|--------|-----------|
| Crear nodo XOR/NAND/NOR/XNOR | Validador | L0 — penalización fuerte |
| Aridad incorrecta | Validador o Parser | L0 / L3 |
| Señal indefinida | Validador o Parser | L0 / L3 |
| Ciclo no permitido | Validador o Parser | L0 / L3 |
| Tabla de verdad incorrecta | Verificador | L1 |
| Propiedad violada | Verificador | L1 |
| Modelo de referencia divergente | Verificador | L1 |
| Caso límite blocking fallido | Verificador | L1 |
| Regresión contra oficial activo | Verificador o Coordinador | L1 / L4 |
| Empate o no-mejora estricta | Evaluador o Coordinador | L2 / L4 |
| Sintaxis o esquema inválido | Parser | L3 |
| Promoción durante conflicto de concurrencia | Coordinador | L4 |
| Tarea con campo prohibido | Cargador de tareas | L5 |

## Tasa de rechazo como métrica de aprendizaje

Las tasas de rechazo por nivel y por agente son **señales legítimas de aprendizaje**:

- **Tasa L0 alta** ⇒ el agente aún no comprende qué acciones son legales en este nivel.
- **Tasa L1 alta** ⇒ el agente construye circuitos que no cumplen la spec.
- **Tasa L2 alta** ⇒ el agente produce soluciones correctas pero subóptimas.
- **Tasa L3 muy alta** ⇒ algo no funciona en el agente o en su generador de `.aoncir`.

El sistema curricular usa estas tasas como input para los criterios de avance (no exclusivos).

## Lo que un rechazo **nunca** implica

- **No es un castigo personal.** Es retroalimentación formal.
- **No reduce el avance curricular ya ganado.**
- **No borra historial del `.aonclg`.** Cada rechazo se registra como aprendizaje.
- **No introduce nuevas primitivas como compensación.** Bajo ninguna circunstancia. Si una tarea es "demasiado dura" porque obliga a construir XOR desde AND/OR/NOT, la solución no es relajar R2, sino dividir la tarea en subtareas más pequeñas.

## Política de re-intento

Tras un rechazo:

- **L0:** el agente prueba otra acción inmediatamente.
- **L1:** el episodio se cierra como fallo; el agente puede reintentar la tarea (nuevo episodio).
- **L2:** el episodio se cierra como solución aceptada pero no promovida.
- **L3:** se reporta el error y el `.aoncir` queda sin cargar.
- **L4:** registro en experimental; el agente puede generar otra versión candidata.
- **L5:** intervención humana sobre el catálogo.

No hay límite duro de reintentos sobre una tarea, pero el sistema curricular registra patrones de fallo persistente.
