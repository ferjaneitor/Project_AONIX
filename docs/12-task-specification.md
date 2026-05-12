# 12 — Especificación formal de tareas

> **Documento normativo.** Define la estructura de toda tarea en AONIX. Cualquier tarea cargada por el coordinador debe ajustarse a esta especificación. El catálogo curricular concreto vive en [06 — Sistema curricular](06-curriculum.md); aquí se fija la **forma** de una tarea.

## Definición

Una **tarea** (`Task`) es la unidad operativa de AONIX. Representa una meta formal verificable que un agente debe alcanzar construyendo un circuito sobre el mundo formal AONIX. Sin tarea no hay episodio. Sin tarea no hay verificación. Sin tarea no hay aprendizaje.

Una tarea **no es** una sugerencia, un ejemplo, ni una solución prefabricada. Es un contrato formal entre el agente y AONIX: el agente se compromete a producir un circuito que cumpla la especificación; AONIX se compromete a verificar y juzgar exclusivamente con los módulos deterministas.

## Estructura formal de una tarea

Toda tarea cumple este esquema. Los campos en **negrita** son obligatorios; los demás opcionales.

```
Task {
    META
    ─────
    id:                  TaskId                # único en el catálogo
    name:                String                # legible
    version:             SemVer                # versionado independiente
    level:               LevelId               # peldaño curricular
    family:              FamilyId?             # familia (ej. sumadores)
    parent:              TaskId?               # tarea prerrequisito directa
    tags:                [String]              # etiquetas no operativas

    DOMINIO
    ───────
    inputs: [
        InputPort {
            name:        Identifier            # único en la tarea
            semantic_tag: SemanticTag?         # ej. operand_bit, carry
            group:       GroupId?              # ej. bus de 4 bits
            arity_bit:   1                     # siempre 1 bit
        }
    ]
    outputs: [
        OutputPort {
            name:        Identifier
            semantic_tag: SemanticTag?         # ej. carry, sum_bit, zero_flag
            group:       GroupId?
            arity_bit:   1
        }
    ]
    semantic_groups: [
        SemanticGroup {
            id:          GroupId
            kind:        bus | address_bus | data_bus | flags | other
            members:     [PortName]
            width:       N                     # bits del grupo
        }
    ]

    META FUNCIONAL
    ──────────────
    spec:                Specification         # ver más abajo
    reference_model:     ReferenceModel?       # software o circuito anterior

    NIVEL Y PRUEBAS
    ───────────────
    required_test_suites: [SuiteRef]           # del repositorio de pruebas
    edge_case_catalog_ref: CatalogRef?         # casos límite a aplicar
    regression_suite_ref: SuiteRef?            # regresión específica
    minimum_random_samples: N                  # cuando aplique
    seed_strategy:       fixed(value) | level_default

    ESTADO INICIAL Y ACCIONES
    ─────────────────────────
    initial_state:       PartialCircuit        # usualmente vacío
    allowed_actions:     [ActionKind]          # subconjunto del nivel
    forbidden_actions:   [ActionKind]          # explícito, refuerzo de R2

    CRITERIOS DE ÉXITO
    ──────────────────
    success_criteria:    SuccessCriteria       # ver más abajo
    failure_criteria:    FailureCriteria       # condiciones de fallo

    EVALUACIÓN ESTRUCTURAL
    ──────────────────────
    evaluated_metrics:   [Metric]              # del evaluador
    metric_thresholds:   {Metric -> Threshold} # umbrales mínimos
    ranking_order:       [Metric]              # orden lexicográfico

    LÍMITES DEL EPISODIO
    ────────────────────
    max_steps:           N
    max_wall_time:       Duration
    abort_on_repeat_invalid: N                 # cortar si bucle de inválidas

    POLÍTICA DE MEMORIA
    ───────────────────
    canonical_promotion: allow | block         # si el cierre puede promover
    historical_archive:  always | on_success   # cuándo archivar

    VISIBILIDAD PARA AGENTE
    ───────────────────────
    visible_to_agent:    AgentVisibilitySet    # ver doc 16
}
```

Cualquier campo desconocido en una tarea cargada por el coordinador la invalida.

## Tipos auxiliares

### `Specification`

La especificación funcional puede expresarse en uno o más de los siguientes formatos. Toda tarea tiene **al menos uno**, y el verificador elige el más fuerte disponible.

```
Specification = OneOf {
    TruthTable {
        rows: [ ([InputBit], [OutputBit]) ]    # tabla exhaustiva
        coverage: complete | partial
    }
  | PropertyList {
        properties: [BooleanProperty]          # cuantificadas sobre entradas
    }
  | ReferenceFunction {
        impl: software_function_id             # función Rust pura referencia
    }
  | ReferenceCircuit {
        circuit_hash: CanonicalHash            # circuito ya verificado oficial
    }
  | TemporalSpec {                              # sólo niveles ≥ 11
        cycles: N
        per_cycle_assertions: [Assertion]
        reset_behavior: ResetSpec
    }
}
```

### `SuccessCriteria`

```
SuccessCriteria {
    verifier_must_pass:           [SuiteRef]   # listado, todas pasan
    minimum_pass_rate_random:     0.0..=1.0    # para pruebas aleatorias
    edge_cases_required_passed:   [CaseId]
    structural_thresholds:        {Metric -> ThresholdComparison}
    semantic_signal_checks:       [SignalCheck] # ej. carry comporta como carry
}
```

Un circuito **solo se considera exitoso** si **todas** las condiciones de `success_criteria` se cumplen. No hay éxito parcial.

### `FailureCriteria`

```
FailureCriteria {
    any_required_suite_fails:     true
    any_edge_case_fails:          true | false  # configurable por tarea
    structural_threshold_breached: true | false
    semantic_signal_violation:    true
    invalid_action_loop_detected: true
    max_steps_exceeded:           true
    max_wall_time_exceeded:       true
}
```

Las condiciones de fallo se evalúan **en orden de aparición**; la primera que se cumple cierra el episodio como fracaso.

### `AgentVisibilitySet`

Lista enumerable de lo que el agente puede observar durante el episodio. Definida en detalle en [16 — Límites de visibilidad para IA](16-ai-visibility-limits.md). Algunos ejemplos:

```
visible:
  - task.meta (id, name, level, family)
  - task.inputs, task.outputs, task.semantic_groups
  - task.spec.truth_table (si la tarea es de nivel donde aplica)
  - task.allowed_actions, task.forbidden_actions
  - episode.partial_circuit
  - simulator.results_on_request
  - validator.feedback_on_each_action
  - evaluator.metrics_on_request
  - reward.partial_running_total

oculto:
  - el circuito final canónico (memoria canónica de la tarea)
  - .aonclg de otros agentes
  - trayectorias de otros agentes
  - versiones históricas, salvo cuando la tarea las habilita explícitamente
  - semilla aleatoria, salvo política de la tarea
```

## Ciclo de vida de una tarea

```
        ┌────────────────────────────────────────────┐
        │            CATÁLOGO                        │
        │  (tareas declaradas y versionadas)         │
        └───────────────┬────────────────────────────┘
                        │ select_task
                        ▼
        ┌────────────────────────────────────────────┐
        │      ACTIVACIÓN POR EPISODIO               │
        │  - cargar Task                             │
        │  - cargar suites de pruebas referenciadas  │
        │  - construir estado inicial                │
        │  - calcular acciones legales iniciales     │
        │  - aplicar AgentVisibilitySet              │
        └───────────────┬────────────────────────────┘
                        │ start
                        ▼
        ┌────────────────────────────────────────────┐
        │           EPISODIO ACTIVO                  │
        │  ciclo: acción → validador → ...           │
        │  (gestionado por el Coordinador, doc 10)   │
        └───────────────┬────────────────────────────┘
                        │ stop_construction | límite
                        ▼
        ┌────────────────────────────────────────────┐
        │       EVALUACIÓN FINAL                     │
        │  - verificador completo                    │
        │  - evaluador completo                      │
        │  - comparación con oficial activo          │
        └───────────────┬────────────────────────────┘
                        │
              ┌─────────┴──────────┐
              ▼                    ▼
        ÉXITO                 FRACASO
        - .aonclg cerrado     - .aonclg cerrado (con causas)
        - quizá promoción     - registro en memoria experimental
        - mem. histórica      - sin promoción
        - mem. optimización
```

## Reglas absolutas que una tarea debe respetar

Una tarea **nunca** puede:

1. **Decir al agente qué compuerta derivada usar.** Una tarea declara la meta, no la solución. Está prohibido que `expected_behavior` incluya frases del tipo "usa XOR para…".
2. **Incluir como `allowed_actions` la creación de un nodo distinto de AND/OR/NOT.** Si lo hace, el cargador de tareas la rechaza.
3. **Marcar como `success_criteria` un valor que no haya emitido el verificador.** No se acepta un atajo por configuración (ej. "se considera correcto si el agente afirma haber terminado").
4. **Eludir el validador.** No existe `bypass_validator: true`.
5. **Modificar `forbidden_actions` para permitir XOR/NAND/NOR/XNOR.** El conjunto base prohibido es **constante del sistema**, y `forbidden_actions` solo puede ser un superconjunto del base, nunca un subconjunto.
6. **Otorgar visibilidad sobre la solución oficial activa** dentro del propio episodio, salvo modos de estudio explícitamente declarados en el currículo (ver doc 16).

## Declaración del catálogo

El catálogo de tareas vive como un conjunto de archivos tipados (formato a decidir, ver [11 — Roadmap](11-roadmap.md)). Cada tarea tiene **id único, versionado semántico y hash canónico** de su contenido.

Modificar una tarea publicada **incrementa la versión**. Las versiones anteriores quedan referenciables (algunos `.aonclg` históricos pueden depender de una versión previa).

## Equivalencia de tareas

Dos tareas son **funcionalmente equivalentes** si tienen la misma `spec`, los mismos puertos (módulo renombramiento), las mismas restricciones absolutas y los mismos criterios de éxito. El sistema detecta equivalencias y las marca para evitar duplicación silenciosa del catálogo.

## Tareas paramétricas

Algunas tareas son **paramétricas** sobre tamaño (`width`). Una sola declaración curricular `full_adder` puede instanciarse para `width = 1, 2, 4, 8, 16, 32`. Cada instancia produce su propia tarea concreta con su `id` derivado (`full_adder.w8`) y su propio `.aoncir` oficial activo por tamaño.

## Tareas con estado temporal

Para tareas de nivel ≥ 11 con `TemporalSpec`, la `spec` se evalúa sobre **secuencias de ciclos**, no sobre vectores únicos. El verificador opera en modo temporal. El simulador reconoce las señales etiquetadas `clock`, `reset`, `enable` y evoluciona el estado discretamente.

Aún en este modo, las primitivas siguen siendo solo AND/OR/NOT. La temporalidad emerge del circuito construido sobre esas primitivas y sobre las señales etiquetadas, **no** de nuevas primitivas.

## Tareas y currículo

El currículo (ver [06](06-curriculum.md)) ordena tareas en niveles. Cada nivel:

- Declara qué clases de tareas existen.
- Declara qué tareas concretas son obligatorias para considerar el nivel "dominado".
- Declara qué tareas son opcionales o de práctica.
- Declara qué tareas desbloquean el siguiente nivel.

Una tarea **pertenece a un único nivel** (su `level`), pero puede aparecer como referencia en múltiples niveles (p. ej. como prerrequisito o como modelo de comparación).

## Pruebas asociadas a una tarea

Cada tarea referencia **explícitamente** las suites que la verifican. El verificador no inventa pruebas: ejecuta lo que la tarea declara. Esto garantiza reproducibilidad: dos agentes resolviendo la misma versión de la misma tarea son juzgados por las mismas pruebas.

La política exacta de pruebas por nivel está en [07 — Pruebas y verificación](07-testing-and-verification.md) y se cruza con la matriz de pruebas por nivel reforzada en ese documento.

## Trazabilidad

Toda activación de una tarea queda registrada con:

- `task.id` y `task.version`.
- `task.hash_canonical` (incluye todo lo del esquema).
- Suites referenciadas con sus versiones y hashes.
- Modelo de referencia con su versión.

Esto permite reconstruir bit a bit cualquier episodio en el futuro.

## Lo que una tarea **no** es

- No es un test unitario. Un test unitario es interno a AONIX; una tarea es la unidad pedagógica.
- No es un ejemplo. Los ejemplos van en documentación; las tareas son contratos.
- No es una solución. Las soluciones viven en memoria canónica (`.aoncir`) y aprendizaje (`.aonclg`).
- No es un atajo. Una tarea no entrega compuertas listas; entrega la meta.
- No es modificable durante el episodio. Una vez activada, su definición es inmutable hasta el cierre.
