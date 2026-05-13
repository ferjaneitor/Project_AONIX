# 20 — Catálogo inicial concreto de tareas (niveles 0–5)

> **Documento normativo.** Define el catálogo concreto de tareas para los niveles 0 a 5 de AONIX. Cada entrada cumple el esquema formal definido en [12 — Especificación formal de tareas](12-task-specification.md). Las pruebas referenciadas siguen la matriz de [07 — Pruebas y verificación](07-testing-and-verification.md). Los criterios de avance siguen la matriz de [06 — Sistema curricular](06-curriculum.md).
>
> **Regla absoluta no negociable:** las tareas pueden pedir construir comportamientos cuya tabla de verdad coincide con XOR, XNOR, NAND, NOR o cualquier otra función booleana derivada, **pero el agente debe construir el circuito usando exclusivamente AND, OR y NOT como compuertas invocables**. Ninguna tarea de este catálogo expone XOR/XNOR/NAND/NOR como compuerta usable, sólo como **meta funcional** que el agente debe lograr componiendo primitivas.

## Convenciones del catálogo

Cada tarea aparece con esta plantilla compacta:

```
id              identificador único (snake_case)
name            nombre legible
level           nivel curricular
inputs          puertos de entrada con etiqueta semántica
outputs         puertos de salida con etiqueta semántica
spec            tabla de verdad o spec resumida
mandatory       true | false  (obligatoria para dominar el nivel)
unlocks         qué tareas o niveles desbloquea
required_tests  suites obligatorias mínimas
success         criterios mínimos de éxito
metrics         métricas evaluadas
notes           comentarios pedagógicos
```

Las primitivas disponibles para todas las tareas son **siempre** `{AND, OR, NOT}`. No se repite en cada entrada.

---

# Nivel 0 — Mundo formal

**Propósito del nivel.** El agente no construye circuitos todavía. Demuestra que comprende el mundo formal: qué es una señal, qué es una compuerta primitiva, qué es un circuito, qué es una tarea, qué es una prueba.

Las tareas de este nivel son **introspectivas**. Las acciones permitidas se limitan a consultas (`request_explanation`, `test_specific_input` sobre circuitos ya existentes preparados como material didáctico).

**Criterio de dominio del nivel:** las cuatro tareas obligatorias resueltas al 100 %.

---

## L0.T1 — `world_identify_gate_types`

```
id              world_identify_gate_types
name            Identificar los tipos de compuerta primitiva
level           0
inputs          (ninguno; introspección)
outputs         (ninguno)
spec            Responder correctamente: el conjunto de primitivas es {AND, OR, NOT}.
mandatory       true
unlocks         world_read_truth_table
required_tests  multiple_choice_suite_v1 (preguntas cerradas con única respuesta)
success         100 % de respuestas correctas
metrics         exactitud
notes           Sirve para que el agente registre R2 desde el primer episodio.
```

## L0.T2 — `world_read_truth_table`

```
id              world_read_truth_table
name            Leer una tabla de verdad de un circuito didáctico
level           0
inputs          circuito didáctico preparado (no se construye; se observa)
outputs         (ninguno)
spec            Dado un .aoncir didáctico simple, leer su tabla de verdad
                (entrada→salida) ejecutando el simulador.
mandatory       true
unlocks         world_simulate_single_input
required_tests  observation_suite_v1
success         El agente reporta la tabla correctamente para 2 ejemplos
                preparados (1 con 1 entrada, 1 con 2 entradas).
metrics         exactitud
notes           El circuito didáctico está pre-cargado y solo se LEE.
```

## L0.T3 — `world_simulate_single_input`

```
id              world_simulate_single_input
name            Simular una entrada específica sobre un circuito didáctico
level           0
inputs          vector de entrada elegido por el agente
outputs         (lectura de la salida producida)
spec            Sobre un circuito didáctico, ejecutar test_specific_input
                con una entrada elegida y reportar la salida.
mandatory       true
unlocks         world_identify_signal_types
required_tests  interactive_suite_v1
success         El agente realiza al menos 4 simulaciones correctas
                (interpreta la salida del simulador).
metrics         exactitud, número de simulaciones realizadas
notes           Introduce la idea de determinismo: misma entrada ⇒ misma salida.
```

## L0.T4 — `world_identify_signal_types`

```
id              world_identify_signal_types
name            Distinguir entrada / salida / señal intermedia
level           0
inputs          circuito didáctico
outputs         (clasificación)
spec            Para cada señal del circuito didáctico, clasificarla como
                puerto_entrada, puerto_salida o señal_intermedia.
mandatory       true
unlocks         nivel 1
required_tests  classification_suite_v1
success         100 % de clasificaciones correctas en 3 circuitos didácticos.
metrics         exactitud
notes           Termina la fundación conceptual antes de construir nada.
```

---

# Nivel 1 — Lógica de una entrada

**Propósito del nivel.** Primeras construcciones. El agente aprende a producir un grafo válido. Espacio de entradas trivial (2 combinaciones).

**Acciones permitidas:** `declare_signal`, `create_gate_NOT`, `assign_output`, `stop_construction`, todas las acciones de introspección.

**Criterio de dominio:** 100 % en las cuatro tareas. Tasa L0 ≤ 20 %.

---

## L1.T1 — `pass_through`

```
id              pass_through
name            Salida igual a la entrada
level           1
inputs          a (semantic_tag: data_bit)
outputs         y (semantic_tag: data_bit)
spec            y = a
                Tabla de verdad:
                    a=0 → y=0
                    a=1 → y=1
mandatory       true
unlocks         inverter
required_tests  exhaustive_suite_2cases
success         Verifier=PASA. Sin señales muertas. Sin compuertas redundantes.
metrics         gate_count (objetivo: 0), depth (objetivo: 0)
notes           Solución óptima: asignación directa entrada→salida.
                Compuertas redundantes (ej. NOT NOT a) deben ser penalizadas
                por el evaluador y eliminadas por el optimizador.
```

## L1.T2 — `inverter`

```
id              inverter
name            Negación
level           1
inputs          a (semantic_tag: data_bit)
outputs         y (semantic_tag: data_bit)
spec            y = NOT a
                Tabla:
                    a=0 → y=1
                    a=1 → y=0
mandatory       true
unlocks         constant_zero, constant_one
required_tests  exhaustive_suite_2cases
success         Verifier=PASA. Solución óptima: 1 NOT.
metrics         gate_count (objetivo: 1 NOT), depth (1)
notes           Primer uso real de una primitiva.
```

## L1.T3 — `constant_zero`

```
id              constant_zero
name            Constante 0
level           1
inputs          a (semantic_tag: data_bit)   (no se usa funcionalmente)
outputs         y (semantic_tag: data_bit)
spec            y = 0 para toda a
mandatory       true
unlocks         constant_one
required_tests  exhaustive_suite_2cases
success         Verifier=PASA. Detección de la entrada como muerta (no usada)
                debe documentarse, no penalizarse (la tarea pide que la entrada
                exista pero permite que no afecte la salida).
metrics         gate_count
notes           Solución canónica de Fase 1:
                  y = a AND NOT a   (equivalente a 0 por complemento booleano).
                AONIX no admite "constante 0 explícita" como SignalReference
                primitiva en Fase 1: la tarea constant_zero es justamente
                aprender a derivar 0 con primitivas. El optimizador puede
                reconocer la equivalencia y reorganizar el grafo, pero la
                versión oficial activa se serializa con compuertas reales.
```

## L1.T4 — `constant_one`

```
id              constant_one
name            Constante 1
level           1
inputs          a (semantic_tag: data_bit)
outputs         y (semantic_tag: data_bit)
spec            y = 1 para toda a
mandatory       true
unlocks         nivel 2
required_tests  exhaustive_suite_2cases
success         Verifier=PASA.
metrics         gate_count
notes           Solución canónica de Fase 1:
                  y = a OR NOT a   (equivalente a 1 por complemento booleano).
                AONIX no admite "constante 1 explícita" como SignalReference
                primitiva en Fase 1: la tarea constant_one obliga a derivar 1
                con primitivas, igual que constant_zero.
```

---

# Nivel 2 — Lógica de dos entradas

**Propósito del nivel.** Resolver **las 16 funciones booleanas de dos entradas como tareas**. Las "compuertas derivadas" (XOR, XNOR, NAND, NOR) aparecen aquí **como comportamientos a construir**, nunca como primitivas invocables. Esta es la lección central del nivel.

**Acciones permitidas:** todas las de construcción combinacional.

**Criterio de dominio:** ≥ 95 % de las 16 funciones (15 o 16). Tasa L0 ≤ 15 %.

---

Cada función tiene su propio identificador `func2_<n>` (índice 0–15 según convención lexicográfica sobre la tabla `(00, 01, 10, 11)`). A continuación se listan las 16. Para no repetir, los campos comunes son:

```
level           2
inputs          a (semantic_tag: data_bit), b (semantic_tag: data_bit)
outputs         y (semantic_tag: data_bit)
required_tests  exhaustive_suite_4cases + edge_cases_2input_v1
success         Verifier=PASA. Sin regresión.
metrics         gate_count, depth, dead_signals
```

| id | nombre | spec | mandatory | notas pedagógicas |
|----|--------|------|-----------|-------------------|
| `func2_0` | constante 0 | y = 0 | true | Mismo que `constant_zero` pero con 2 entradas. |
| `func2_1` | AND | y = a AND b | true | 1 AND. |
| `func2_2` | inhibición a→b | y = a AND NOT b | true | Función de inhibición. |
| `func2_3` | proyección a | y = a | true | Salida = entrada A. |
| `func2_4` | inhibición b→a | y = NOT a AND b | true | |
| `func2_5` | proyección b | y = b | true | Salida = entrada B. |
| `func2_6` | **comportamiento XOR** | y = (a AND NOT b) OR (NOT a AND b) | true | **Tarea clave: XOR como composición.** Nunca como primitiva. |
| `func2_7` | OR | y = a OR b | true | 1 OR. |
| `func2_8` | **comportamiento NOR** | y = NOT (a OR b) | true | NOR como composición. |
| `func2_9` | **comportamiento XNOR** | y = NOT ((a AND NOT b) OR (NOT a AND b)) | true | XNOR como composición. |
| `func2_10` | negación de b | y = NOT b | true | |
| `func2_11` | implicación b→a | y = a OR NOT b | true | |
| `func2_12` | negación de a | y = NOT a | true | |
| `func2_13` | implicación a→b | y = NOT a OR b | true | |
| `func2_14` | **comportamiento NAND** | y = NOT (a AND b) | true | NAND como composición. |
| `func2_15` | constante 1 | y = 1 | true | |

**Casos límite obligatorios para todas:** `(0,0)`, `(0,1)`, `(1,0)`, `(1,1)` (exhaustiva cubre todos), más entradas declaradas como casos de regresión por familia (catálogo, ver [22](22-edge-case-catalog.md)).

**Reemplazo a oficial activo:** desde este nivel, una solución correcta puede competir por oficial activo de su tarea. Hash canónico determina identidad. Una solución estructuralmente equivalente a la oficial no la reemplaza (empate ⇒ se queda el incumbente).

---

# Nivel 3 — Tres o más entradas

**Propósito del nivel.** Aumentar la combinatoria. Aparecen primeros casos donde la solución se beneficia de reutilizar subexpresiones internas. Las pruebas siguen siendo exhaustivas (≤ 16 combinaciones).

**Criterio de dominio:** ≥ 90 %. Tasa L0 ≤ 15 %. Casos límite del nivel todos superados.

---

## L3.T1 — `and_3`

```
id              and_3
name            AND de tres entradas
level           3
inputs          a, b, c (semantic_tag: data_bit cada una)
outputs         y (semantic_tag: data_bit)
spec            y = a AND b AND c
                Verdadera si y solo si las tres son 1.
mandatory       true
unlocks         or_3
required_tests  exhaustive_suite_8cases + edge_cases_3input_v1
success         Verifier=PASA.
metrics         gate_count, depth
notes           Solución óptima: composición de 2 AND binarios
                (por ejemplo, t = a AND b; y = t AND c).
                AONIX usa aridad estricta binaria en Fase 1 para AND y OR
                (ver docs/01-rules-absolute.md). Un "AND de 3 entradas" se
                construye encadenando dos AND, lo cual cuenta como dos
                gates en las métricas — comportamiento honesto deseado.
```

## L3.T2 — `or_3`

```
id              or_3
name            OR de tres entradas
level           3
inputs          a, b, c
outputs         y
spec            y = a OR b OR c
mandatory       true
unlocks         majority_3
required_tests  exhaustive_suite_8cases + edge_cases_3input_v1
success         Verifier=PASA.
metrics         gate_count, depth
notes           —
```

## L3.T3 — `majority_3`

```
id              majority_3
name            Mayoría de tres
level           3
inputs          a, b, c
outputs         y
spec            y = 1 si y solo si al menos 2 de {a, b, c} son 1
                Equivalente a (a AND b) OR (a AND c) OR (b AND c)
                o a (a AND b) OR (c AND (a OR b))
mandatory       true
unlocks         parity_3
required_tests  exhaustive_suite_8cases
                + edge_cases_3input_v1
                + property_majority_self_dual
success         Verifier=PASA. Mejora respecto a versión histórica si la hay.
metrics         gate_count, depth, reuse_score
notes           Buen ejemplo de tarea donde aparece reutilización: la señal
                (a AND b) puede compartirse en dos términos.
```

## L3.T4 — `parity_3`

```
id              parity_3
name            Paridad de tres (XOR de tres como comportamiento)
level           3
inputs          a, b, c
outputs         y (semantic_tag: parity_bit)
spec            y = a XOR b XOR c   (entendido como comportamiento)
                Equivalente: y = 1 si número impar de entradas a 1.
                Construcción típica con primitivas:
                    t1 = (a AND NOT b) OR (NOT a AND b)
                    y  = (t1 AND NOT c) OR (NOT t1 AND c)
mandatory       true
unlocks         detect_pattern_111
required_tests  exhaustive_suite_8cases
                + edge_cases_3input_v1
success         Verifier=PASA. Sin uso de primitivas prohibidas
                (verificación automática del validador).
metrics         gate_count, depth
notes           Tarea pivote: el agente debe descubrir cómo encadenar dos
                "XORs como composición" sin que aparezca XOR como primitiva
                en su .aoncir.
```

## L3.T5 — `detect_pattern_111`

```
id              detect_pattern_111
name            Detector del patrón 1-1-1
level           3
inputs          a, b, c
outputs         y (semantic_tag: pattern_match)
spec            y = 1 si y solo si (a, b, c) = (1, 1, 1)
mandatory       true
unlocks         detect_pattern_010
required_tests  exhaustive_suite_8cases
success         Verifier=PASA. Soluciones tipo (a AND b AND c) son óptimas.
metrics         gate_count, depth
notes           Caso trivial; sirve para mecanismo de detección de patrones.
```

## L3.T6 — `detect_pattern_010`

```
id              detect_pattern_010
name            Detector del patrón 0-1-0
level           3
inputs          a, b, c
outputs         y (semantic_tag: pattern_match)
spec            y = 1 si y solo si (a, b, c) = (0, 1, 0)
                Equivalente: y = NOT a AND b AND NOT c
mandatory       true
unlocks         nivel 4
required_tests  exhaustive_suite_8cases
success         Verifier=PASA.
metrics         gate_count, depth
notes           Introduce el uso combinado de NOT y AND para detectar
                patrones específicos.
```

---

# Nivel 4 — Multi-salida

**Propósito del nivel.** Construir circuitos con **varias salidas que comparten subexpresiones internas**. La validación incremental se vuelve obligatoria. Aparece la verificación por señal semántica (cada salida tiene su propia spec).

**Criterio de dominio:** ≥ 90 %. Tasa L0 ≤ 15 %. Reutilización demostrada (al menos una señal interna usada por más de una salida en al menos 3 de las soluciones del nivel).

---

## L4.T1 — `half_adder`

```
id              half_adder
name            Half adder
level           4
inputs          a (operand_bit), b (operand_bit)
outputs         sum (semantic_tag: sum_bit), carry (semantic_tag: carry)
spec            sum   = (a AND NOT b) OR (NOT a AND b)   [comportamiento XOR]
                carry = a AND b
                Tabla:
                    (0,0) → (sum=0, carry=0)
                    (0,1) → (sum=1, carry=0)
                    (1,0) → (sum=1, carry=0)
                    (1,1) → (sum=0, carry=1)
mandatory       true
unlocks         comparator_1bit
required_tests  exhaustive_suite_4cases
                + edge_cases_2input_v1
                + semantic_signal_check.carry
                + property_sum_symmetry
success         Verifier=PASA en ambas salidas.
metrics         gate_count, depth, reuse_score (alto valor si comparte señales)
notes           Tarea clásica para introducir reutilización: la señal
                "a AND b" puede compartirse entre carry y la negación útil
                para sum. La etiqueta carry exige verificación semántica.
```

## L4.T2 — `comparator_1bit`

```
id              comparator_1bit
name            Comparador de 1 bit
level           4
inputs          a (operand_bit), b (operand_bit)
outputs         eq (semantic_tag: comparison),
                gt (semantic_tag: comparison),
                lt (semantic_tag: comparison)
spec            eq = (a AND b) OR (NOT a AND NOT b)
                gt = a AND NOT b
                lt = NOT a AND b
mandatory       true
unlocks         decoder_2_to_4
required_tests  exhaustive_suite_4cases
                + property_exactly_one_of_eq_gt_lt
                + semantic_signal_check.comparison
success         Verifier=PASA. Propiedad universal: exactamente una de
                {eq, gt, lt} es 1 en cada vector.
metrics         gate_count, depth, reuse_score
notes           Buen ejemplo de tres salidas que comparten subexpresiones
                (NOT a y NOT b se reutilizan).
```

## L4.T3 — `decoder_2_to_4`

```
id              decoder_2_to_4
name            Decodificador 2 a 4
level           4
inputs          s0 (semantic_tag: select), s1 (semantic_tag: select)
outputs         y0, y1, y2, y3 (semantic_tag: select_output cada una)
spec            y0 = NOT s1 AND NOT s0
                y1 = NOT s1 AND s0
                y2 = s1 AND NOT s0
                y3 = s1 AND s0
                Exactamente una salida es 1 en cada vector.
mandatory       true
unlocks         mux_2_to_1, demux_1_to_2
required_tests  exhaustive_suite_4cases
                + property_one_hot_output
                + semantic_signal_check.select
success         Verifier=PASA. Propiedad universal: salida one-hot.
metrics         gate_count, depth, reuse_score
notes           Las señales NOT s0 y NOT s1 se reutilizan; la solución
                óptima minimiza inversores.
```

## L4.T4 — `mux_2_to_1`

```
id              mux_2_to_1
name            Multiplexor 2 a 1 (versión nivel 4: aún no canónico)
level           4
inputs          d0 (data_bit), d1 (data_bit), s (select)
outputs         y (data_bit)
spec            y = (NOT s AND d0) OR (s AND d1)
                Selecciona d0 si s=0; d1 si s=1.
mandatory       true
unlocks         demux_1_to_2
required_tests  exhaustive_suite_8cases
                + property_mux_select_behavior
                + edge_cases_3input_v1
                + semantic_signal_check.select
success         Verifier=PASA. Propiedad universal: si s=0 ⇒ y=d0; si s=1 ⇒ y=d1.
metrics         gate_count, depth
notes           Versión "no canónica" del mux. Sube a canónico en el
                nivel 5 con nombre formal multiplexer_2_to_1.aoncir.
```

## L4.T5 — `demux_1_to_2`

```
id              demux_1_to_2
name            Demultiplexor 1 a 2 (versión nivel 4)
level           4
inputs          d (data_bit), s (select)
outputs         y0 (data_bit), y1 (data_bit)
spec            y0 = NOT s AND d
                y1 = s AND d
mandatory       true
unlocks         nivel 5
required_tests  exhaustive_suite_4cases
                + property_only_one_output_active_when_d_is_1
                + semantic_signal_check.select
success         Verifier=PASA.
metrics         gate_count, depth, reuse_score
notes           Demux como inverso conceptual del mux.
```

---

# Nivel 5 — Circuitos nombrados canónicos

**Propósito del nivel.** El agente produce circuitos que ganan **nombre canónico** y entran a memoria canónica como `.aoncir` oficiales activos por (name, parameters). Aparece la comparación con histórico y la promoción atómica.

**Criterio de dominio:** ≥ 90 %. Tasa L0 ≤ 12 %. Explicación humana recomendable. Todas las tareas obligatorias deben haber pasado por promoción al menos una vez.

---

## L5.T1 — `multiplexer_2_to_1` (canónico)

```
id              multiplexer_2_to_1
canonical_name  multiplexer_2_to_1
level           5
inputs          d0 (data_bit), d1 (data_bit), s (select)
outputs         y (data_bit)
spec            y = (NOT s AND d0) OR (s AND d1)
mandatory       true
unlocks         multiplexer_4_to_1
required_tests  exhaustive_suite_8cases
                + property_mux_select_behavior
                + regression_suite_mux_family_v1
                + semantic_signal_check.select
success         Verifier=PASA. Promoción a memoria canónica posible.
metrics         gate_count, depth, reuse_score, complexity_visual
ranking_order   [gate_count, depth, reuse_score, fan_out_max]
notes           Primera entidad canónica formal de AONIX. Su .aoncir
                permanece expandido a AND/OR/NOT.
                NO se introduce ninguna primitiva "MUX" en el sistema.
```

## L5.T2 — `multiplexer_4_to_1` (canónico)

```
id              multiplexer_4_to_1
canonical_name  multiplexer_4_to_1
level           5
inputs          d0, d1, d2, d3 (data_bit),
                s0, s1 (select; grupo "select_bus", width=2)
outputs         y (data_bit)
spec            Selección de uno de {d0..d3} según (s1, s0):
                    (0,0) → y = d0
                    (0,1) → y = d1
                    (1,0) → y = d2
                    (1,1) → y = d3
                Equivalente a y =  (NOT s1 AND NOT s0 AND d0)
                              OR (NOT s1 AND     s0 AND d1)
                              OR (    s1 AND NOT s0 AND d2)
                              OR (    s1 AND     s0 AND d3)
mandatory       true
unlocks         demultiplexer_1_to_2
required_tests  exhaustive_suite_64cases
                + property_mux_select_behavior_2bit
                + regression_suite_mux_family_v1
                + semantic_signal_check.select_bus
success         Verifier=PASA. Solución expandida a AND/OR/NOT.
                Reutilización de NOT s0 y NOT s1 esperada.
metrics         gate_count, depth, reuse_score
ranking_order   [gate_count, depth, reuse_score]
notes           Introduce el concepto de bus (grupo "select_bus") como
                etiqueta semántica, no como compuerta nueva.
                Ver doc 24 para convenciones de buses.
```

## L5.T3 — `demultiplexer_1_to_2` (canónico)

```
id              demultiplexer_1_to_2
canonical_name  demultiplexer_1_to_2
level           5
inputs          d (data_bit), s (select)
outputs         y0 (data_bit), y1 (data_bit)
spec            y0 = NOT s AND d
                y1 = s AND d
mandatory       true
unlocks         one_bit_full_adder
required_tests  exhaustive_suite_4cases
                + property_only_one_output_active_when_d_is_1
                + regression_suite_demux_family_v1
success         Verifier=PASA. Promoción posible.
metrics         gate_count, depth
ranking_order   [gate_count, depth]
notes           Versión canónica de la tarea L4.T5.
```

## L5.T4 — `one_bit_full_adder` (canónico)

```
id              one_bit_full_adder
canonical_name  one_bit_full_adder
level           5
parameters      width = 1
inputs          a (operand_bit), b (operand_bit), cin (carry)
outputs         sum (sum_bit), cout (carry)
spec            sum  = a XOR b XOR cin                 [comportamiento]
                cout = (a AND b) OR (cin AND (a XOR b))
                       o equivalentemente
                cout = (a AND b) OR (a AND cin) OR (b AND cin)
                Tabla de verdad: 8 combinaciones.
                Solución canónica completamente expandida a AND/OR/NOT.
mandatory       true
unlocks         comparator_2bit
required_tests  exhaustive_suite_8cases
                + edge_cases_3input_v1
                + property_full_adder_sum_arithmetic
                + property_full_adder_carry_arithmetic
                + regression_suite_adder_family_v1
                + semantic_signal_check.carry
                + reference_model.full_adder_1bit_software
success         Verifier=PASA en sum y cout. Coincidencia con modelo
                aritmético software. Sin regresión contra oficial activo.
metrics         gate_count, depth, reuse_score
ranking_order   [gate_count, depth, reuse_score, fan_out_max]
notes           Pieza central del catálogo. Reutilizable como subcomponente
                conceptual (no como primitiva opaca) en sumadores N-bit
                del nivel 8.
                Pedagógicamente: el agente debe DESCUBRIR que la señal
                (a AND NOT b) OR (NOT a AND b) — el XOR como comportamiento —
                aparece dos veces y puede compartirse.
```

## L5.T5 — `comparator_2bit` (canónico)

```
id              comparator_2bit
canonical_name  comparator_2bit
level           5
parameters      width = 2
inputs          a0, a1 (operand_bit, grupo "operand_a", width=2),
                b0, b1 (operand_bit, grupo "operand_b", width=2)
outputs         eq (comparison), gt (comparison), lt (comparison)
spec            eq = (a1 == b1) AND (a0 == b0)
                gt = (a1 AND NOT b1) OR ((a1 == b1) AND (a0 AND NOT b0))
                lt = (NOT a1 AND b1) OR ((a1 == b1) AND (NOT a0 AND b0))
                expandido a AND/OR/NOT.
mandatory       true
unlocks         decoder_3_to_8
required_tests  exhaustive_suite_16cases
                + property_exactly_one_of_eq_gt_lt
                + regression_suite_comparator_family_v1
                + semantic_signal_check.comparison
                + reference_model.compare_2bit_software
success         Verifier=PASA. Propiedad de exclusión mutua de eq/gt/lt.
metrics         gate_count, depth, reuse_score
ranking_order   [gate_count, depth, reuse_score]
notes           Introduce comparación multi-bit. La señal "iguales por bit"
                debe poder reutilizarse entre eq, gt, lt.
```

## L5.T6 — `decoder_3_to_8` (canónico)

```
id              decoder_3_to_8
canonical_name  decoder_3_to_8
level           5
inputs          s0, s1, s2 (select; grupo "select_bus", width=3)
outputs         y0..y7 (select_output cada una)
spec            yi = 1 si y solo si (s2, s1, s0) en binario = i.
                One-hot output sobre 8 salidas.
mandatory       true
unlocks         nivel 6
required_tests  exhaustive_suite_8cases
                + property_one_hot_output
                + regression_suite_decoder_family_v1
                + semantic_signal_check.select_bus
success         Verifier=PASA en las 8 salidas. Propiedad universal one-hot.
metrics         gate_count, depth, reuse_score
ranking_order   [gate_count, depth, reuse_score]
notes           Última tarea obligatoria del nivel 5. Cierra el dominio
                combinacional clásico antes de pasar a buses (nivel 6) y
                aritmética (nivel 7).
                Las señales NOT s0, NOT s1, NOT s2 son altamente reutilizables.
```

---

## Pseudo-resumen del catálogo

| Nivel | Tareas obligatorias | Total |
|------|---------------------|-------|
| 0 | world_identify_gate_types, world_read_truth_table, world_simulate_single_input, world_identify_signal_types | 4 |
| 1 | pass_through, inverter, constant_zero, constant_one | 4 |
| 2 | func2_0 … func2_15 (16 funciones, incluye XOR/NOR/XNOR/NAND como comportamientos) | 16 |
| 3 | and_3, or_3, majority_3, parity_3, detect_pattern_111, detect_pattern_010 | 6 |
| 4 | half_adder, comparator_1bit, decoder_2_to_4, mux_2_to_1, demux_1_to_2 | 5 |
| 5 | multiplexer_2_to_1, multiplexer_4_to_1, demultiplexer_1_to_2, one_bit_full_adder, comparator_2bit, decoder_3_to_8 | 6 |
| **Total** | | **41** |

## Tareas opcionales sugeridas (no obligatorias)

Pueden añadirse para enriquecer el corpus de aprendizaje, sin cambiar la condición de avance:

- `identity_2input` (y = a, ignorando b — variante de proyección).
- `sum_only_no_carry` (medio del half adder).
- `priority_encoder_4_to_2` (avanzada, nivel 5 opcional).
- `binary_to_unary_2bit` (variante de decoder).

## Garantías del catálogo

1. **Ninguna tarea introduce XOR/XNOR/NAND/NOR como compuerta invocable.** Aparecen exclusivamente como **comportamiento esperado** que el agente compone a partir de AND/OR/NOT.
2. **Ningún `expected_behavior` instruye al agente a "usar XOR".** La meta se declara como tabla de verdad o equivalencia algebraica con primitivas; el "qué" lógico es responsabilidad del agente.
3. **Toda tarea respeta el esquema de [12](12-task-specification.md).**
4. **Toda tarea referencia suites de pruebas existentes o por existir; la creación de esas suites es prerequisito de Fase 1.**
5. **Ninguna tarea otorga visibilidad sobre el oficial activo de la misma tarea durante el episodio activo.** Modos de estudio solo post-cierre, según [16](16-ai-visibility-limits.md).

## Trazabilidad

Toda activación de una tarea de este catálogo queda registrada con:

- `task.id` y `task.version` (este documento es versión 1.0.0).
- `task.hash_canonical` (a calcular cuando el catálogo se serialice).
- Suites y modelos de referencia con sus hashes.

## Decisiones pendientes específicas del catálogo

- Identidad exacta de cada suite referenciada (`exhaustive_suite_4cases`, `regression_suite_mux_family_v1`, etc.): los **nombres** son normativos; las **definiciones físicas** (qué casos concretos contiene cada suite) se cierran cuando se materialice la memoria de pruebas en Fase 1.
- Catálogo de etiquetas semánticas referenciadas (`data_bit`, `operand_bit`, `carry`, `select`, etc.) — fijado en [24 — Convenciones de etiquetas semánticas](24-semantic-tag-conventions.md).
- Propiedades como `property_sum_symmetry`, `property_one_hot_output`, `property_mux_select_behavior` — sus **enunciados** se redactarán al consolidar la suite de pruebas en Fase 1; aquí solo se referencian por nombre.

## Lo que el catálogo **no** define

- La sintaxis física del `.aoncir` (ver [21](21-aoncir-syntax.md)).
- Casos límite específicos por familia (ver [22](22-edge-case-catalog.md)).
- Transformaciones del optimizador (ver [23](23-optimizer-transformations.md)).
- Implementación de las suites (Fase 1).
- Implementación de los modelos de referencia (Fase 1).
