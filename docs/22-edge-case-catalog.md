# 22 — Catálogo de casos límite por familia de circuito

> **Documento normativo.** Define el catálogo inicial de casos límite que el verificador aplica por familia de circuito. Cada caso es **inmutable una vez añadido**; las suites son append-only (ver [07 — Pruebas y verificación](07-testing-and-verification.md), [19 — Política de versionado](19-versioning-policy.md)).

## Estructura de una entrada

Cada caso límite cumple esta plantilla:

```
id              identificador único snake_case
family          familia a la que aplica
applies_to      lista de id de tareas o patrón
inputs          vector(es) concreto(s) de entrada
expected        comportamiento esperado o referencia
blocking        true | false  — si su fallo bloquea la aceptación
rationale       motivo de por qué este caso es importante
introduced_in   versión del catálogo en que se incorporó
```

Una vez añadido, un caso límite **no se elimina** sin auditoría humana (ver [25](25-human-audit-policy.md) §"Cambios de pruebas").

## Categorías de casos límite (transversales)

Antes de las familias, hay categorías universales aplicables a todo circuito combinacional:

### Universales transversales

```
id              edge_all_zeros
family          (todas las combinacionales)
inputs          todas las entradas = 0
expected        según spec
blocking        true
rationale       Detecta circuitos que solo funcionan con al menos una entrada activa.

id              edge_all_ones
family          (todas las combinacionales)
inputs          todas las entradas = 1
expected        según spec
blocking        true
rationale       Detecta circuitos con saturación o fan-out incorrecto.

id              edge_single_bit_on
family          (todas las combinacionales con ≥ 2 entradas)
inputs          un solo bit a 1, los demás a 0; iterado por cada bit
expected        según spec
blocking        true
rationale       Verifica que cada entrada contribuye independientemente.

id              edge_single_bit_off
family          (todas las combinacionales con ≥ 2 entradas)
inputs          un solo bit a 0, los demás a 1; iterado por cada bit
expected        según spec
blocking        true
rationale       Dual de single_bit_on.

id              edge_alternating_pattern_01
family          (todas las combinacionales con ≥ 2 entradas)
inputs          patrón alternante 0,1,0,1,...
expected        según spec
blocking        false en niveles ≤ 3, true en niveles ≥ 4
rationale       Detecta dependencias entre bits adyacentes.

id              edge_alternating_pattern_10
family          (todas las combinacionales con ≥ 2 entradas)
inputs          patrón alternante 1,0,1,0,...
expected        según spec
blocking        false en niveles ≤ 3, true en niveles ≥ 4
rationale       Dual del anterior.
```

---

## Familia: lógica booleana simple (niveles 1–3)

Tareas cubiertas: `pass_through`, `inverter`, `constant_zero`, `constant_one`, `func2_*`, `and_3`, `or_3`, `majority_3`, `parity_3`, `detect_pattern_*`.

```
id              edge_logic_constant_input_assumption_break
family          logic_basic
applies_to      tareas con entrada cuyo papel sería ignorable (constantes)
inputs          forzar la entrada "ignorable" a oscilar entre 0 y 1
expected        salida idéntica para ambos valores cuando la spec lo dice
blocking        true
rationale       Detecta circuitos que indebidamente acoplan una entrada
                irrelevante a la salida.

id              edge_logic_complementary_pair
family          logic_basic (≥ 2 entradas)
applies_to      todas las func2_*
inputs          (a, b) = (0,1) y (1,0)
expected        según tabla de verdad
blocking        true
rationale       Casos donde XOR/XNOR como comportamiento se distinguen
                fácilmente de AND/OR.

id              edge_logic_xor_corner
family          logic_basic
applies_to      func2_6 (comportamiento XOR), func2_9 (comportamiento XNOR),
                parity_3
inputs          todas las combinaciones de paridad par y de paridad impar
expected        comportamiento XOR/XNOR/paridad correcto
blocking        true
rationale       Caso clásico donde fallos silenciosos producen tablas
                "casi correctas".

id              edge_logic_nand_corner
family          logic_basic
applies_to      func2_14 (comportamiento NAND), func2_8 (NOR)
inputs          (0,0), (1,1) y los intermedios
expected        salida correcta según spec
blocking        true
rationale       Verifica los puntos de transición que más fallan en
                composiciones erróneas.

id              edge_logic_majority_tie
family          logic_basic
applies_to      majority_3
inputs          (1,1,0), (1,0,1), (0,1,1) (mayoría justa)
expected        y = 1
blocking        true
rationale       Casos en el límite exacto del umbral de mayoría.

id              edge_logic_majority_below
family          logic_basic
applies_to      majority_3
inputs          (1,0,0), (0,1,0), (0,0,1) (por debajo del umbral)
expected        y = 0
blocking        true
rationale       Dual del anterior.
```

---

## Familia: circuitos multi-salida (nivel 4)

Tareas cubiertas: `half_adder`, `comparator_1bit`, `decoder_2_to_4`, `mux_2_to_1`, `demux_1_to_2`.

```
id              edge_multi_one_output_at_a_time
family          multi_output
applies_to      decoder_2_to_4, decoder_3_to_8, demux_1_to_2
inputs          cada combinación de entrada
expected        propiedad one-hot: exactamente UNA salida activa
blocking        true
rationale       Propiedad universal del decoder; fallar significa
                acoplo incorrecto entre salidas.

id              edge_multi_mutually_exclusive
family          multi_output
applies_to      comparator_1bit, comparator_2bit
inputs          todas las combinaciones
expected        propiedad de exclusión: eq XOR gt XOR lt = 1
                (exactamente una activa)
blocking        true
rationale       Si dos salidas se activan simultáneamente o ninguna,
                el comparador está roto.

id              edge_multi_carry_consistency
family          multi_output
applies_to      half_adder
inputs          (0,0), (0,1), (1,0), (1,1)
expected        sum y carry consistentes con suma aritmética 2*carry + sum = a + b
blocking        true
rationale       Verifica que sum y carry forman juntos la suma real.
```

---

## Familia: multiplexores

Tareas cubiertas: `mux_2_to_1`, `multiplexer_2_to_1`, `multiplexer_4_to_1`.

```
id              edge_mux_select_zero
family          mux
applies_to      multiplexer_2_to_1, multiplexer_4_to_1
inputs          select = 0 (o select_bus = 00...0); datos cualesquiera
expected        y = d0
blocking        true
rationale       Propiedad fundamental del mux.

id              edge_mux_select_max
family          mux
applies_to      multiplexer_4_to_1
inputs          select_bus = 11; datos cualesquiera
expected        y = d3
blocking        true

id              edge_mux_data_ignored
family          mux
applies_to      multiplexer_*
inputs          mismo select, datos no-seleccionados varían arbitrariamente
expected        y depende SOLO del dato seleccionado
blocking        true
rationale       Verifica aislamiento entre canales de datos no seleccionados.

id              edge_mux_all_data_zero
family          mux
applies_to      multiplexer_*
inputs          todos los datos = 0
expected        y = 0 para todo select
blocking        true

id              edge_mux_all_data_one
family          mux
applies_to      multiplexer_*
inputs          todos los datos = 1
expected        y = 1 para todo select
blocking        true

id              edge_mux_select_glitch_simulated
family          mux
applies_to      multiplexer_* (en modo no temporal: como propiedad combinacional)
inputs          select va de un valor a otro; ambos datos seleccionados
                comparten el mismo valor
expected        y permanece igual (sin "glitch" combinacional simulado)
blocking        false (recomendado)
rationale       Propiedad estructural deseable; en modo combinacional
                puro es siempre cierta porque solo importa el valor final.
                Se cataloga para reuso futuro en niveles temporales.
```

---

## Familia: comparadores

Tareas cubiertas: `comparator_1bit`, `comparator_2bit` y futuros N-bit.

```
id              edge_cmp_equal
family          cmp
applies_to      comparator_*
inputs          a == b para varios valores
expected        eq = 1, gt = 0, lt = 0
blocking        true

id              edge_cmp_greater
family          cmp
applies_to      comparator_*
inputs          a > b
expected        eq = 0, gt = 1, lt = 0
blocking        true

id              edge_cmp_lesser
family          cmp
applies_to      comparator_*
inputs          a < b
expected        eq = 0, gt = 0, lt = 1
blocking        true

id              edge_cmp_min_vs_max
family          cmp
applies_to      comparator_2bit (y N-bit)
inputs          a = todos ceros, b = todos unos; y viceversa
expected        gt o lt correctos según orientación
blocking        true

id              edge_cmp_adjacent_values
family          cmp
applies_to      comparator_2bit (y N-bit)
inputs          a y b adyacentes (b = a + 1, b = a - 1)
expected        gt o lt correctos
blocking        true
rationale       Verifica que el comparador detecta diferencias pequeñas.

id              edge_cmp_zero_against_one
family          cmp
applies_to      comparator_*
inputs          a = 0, b = 1
expected        lt = 1
blocking        true
```

---

## Familia: sumadores

Tareas cubiertas: `half_adder`, `one_bit_full_adder`, `two_bit_full_adder`, ... `thirty_two_bit_full_adder`.

```
id              edge_add_no_carry_in
family          adder
applies_to      *_full_adder
inputs          cin = 0; a y b varían
expected        sum y cout coincidentes con (a + b)
blocking        true

id              edge_add_with_carry_in
family          adder
applies_to      *_full_adder
inputs          cin = 1; a y b varían
expected        sum y cout coincidentes con (a + b + 1)
blocking        true

id              edge_add_carry_propagation_chain
family          adder
applies_to      sumadores N-bit
inputs          a = todos unos, b = un solo bit en posición 0, cin = 0
expected        carry se propaga a través de toda la cadena
blocking        true
rationale       Caso histórico clásico de bug por ripple carry incorrecto.

id              edge_add_max_plus_max
family          adder
applies_to      sumadores N-bit
inputs          a = max, b = max (todos unos)
expected        sum = 2^N - 2, cout = 1
blocking        true
rationale       Overflow máximo unsigned.

id              edge_add_max_plus_one
family          adder
applies_to      sumadores N-bit
inputs          a = max, b = 1
expected        sum = 0, cout = 1
blocking        true
rationale       Overflow exacto a cero.

id              edge_add_zero_plus_zero_plus_cin
family          adder
applies_to      *_full_adder
inputs          a = 0, b = 0, cin = 1
expected        sum = 1, cout = 0
blocking        true
rationale       El acarreo de entrada por sí solo debe propagar a sum.

id              edge_add_signed_overflow_positive
family          adder
applies_to      sumadores N-bit con bandera overflow signed (niveles ≥ 9)
inputs          a y b ambos positivos en complemento a dos, resultado > MAX_INT
expected        overflow_flag = 1
blocking        true (cuando aplique)

id              edge_add_signed_overflow_negative
family          adder
applies_to      sumadores N-bit con bandera overflow signed (niveles ≥ 9)
inputs          a y b ambos negativos en complemento a dos, resultado < MIN_INT
expected        overflow_flag = 1
blocking        true (cuando aplique)
```

---

## Familia: buses pequeños

Tareas cubiertas: `multiplexer_4_to_1` (select_bus de 2), buses de operandos en sumadores y comparadores, `decoder_3_to_8`.

```
id              edge_bus_all_zero
family          bus_small
applies_to      cualquier circuito que reciba bus etiquetado
inputs          el bus completo = todos ceros
expected        según spec
blocking        true
rationale       Caso límite "no input" via bus.

id              edge_bus_all_one
family          bus_small
applies_to      cualquier circuito que reciba bus etiquetado
inputs          el bus completo = todos unos
expected        según spec
blocking        true

id              edge_bus_single_hot
family          bus_small
applies_to      decoder_2_to_4, decoder_3_to_8, mux con bus de select
inputs          bus con exactamente un bit a 1
expected        decoder one-hot; mux selecciona la entrada correspondiente
blocking        true
rationale       Caso clásico de operación correcta del decoder.

id              edge_bus_walking_one
family          bus_small
applies_to      decoders y muxes con bus de select de N bits
inputs          serie de vectores donde solo un bit a la vez está en 1,
                iterando por todos los bits
expected        salida correspondiente activa
blocking        true
rationale       Verifica cobertura uno a uno del decoder.

id              edge_bus_walking_zero
family          bus_small
applies_to      idem
inputs          serie de vectores donde solo un bit a la vez está en 0
expected        según spec
blocking        false en niveles 6, true en niveles 8+
rationale       Dual del walking one.

id              edge_bus_endianness_consistency
family          bus_small
applies_to      sumadores N-bit, comparadores N-bit
inputs          mismo valor entero presentado con orden de bits explícito
expected        resultado consistente con la convención LSB-first canónica
                de AONIX (bit_position = 0 es LSB, ver docs/24 §U.7)
blocking        true
rationale       Bug histórico: confundir LSB con MSB. AONIX cierra esta
                vía fijando la convención por reglamento, no por tarea.
```

---

## Familia: flags

Tareas cubiertas: salidas etiquetadas `zero_flag`, `carry_flag`, `overflow_flag`, `negative_flag` (a partir del nivel 9; previstas en futuro).

```
id              edge_flag_zero_exact
family          flags
applies_to      circuitos con zero_flag
inputs          combinación que produce resultado = 0
expected        zero_flag = 1, demás flags según spec
blocking        true

id              edge_flag_zero_near_miss
family          flags
applies_to      circuitos con zero_flag
inputs          combinación que produce resultado ≠ 0 (cualquiera)
expected        zero_flag = 0
blocking        true

id              edge_flag_carry_exact_threshold
family          flags
applies_to      sumadores con carry_flag
inputs          a + b = exactamente 2^N (overflow unsigned mínimo)
expected        carry_flag = 1, sum = 0
blocking        true

id              edge_flag_overflow_exact_signed
family          flags
applies_to      sumadores/restadores con overflow_flag (signed)
inputs          combinaciones que cruzan exactamente la frontera
                signed: MAX_INT + 1, MIN_INT - 1
expected        overflow_flag = 1
blocking        true

id              edge_flag_negative_bit
family          flags
applies_to      operaciones con resultado signed y negative_flag
inputs          combinaciones que producen MSB = 1
expected        negative_flag = 1
blocking        true

id              edge_flag_no_false_positives
family          flags
applies_to      cualquier circuito con flags
inputs          un muestreo aleatorio reproducible de combinaciones
                que NO disparan la flag según spec
expected        flag = 0 en todas
blocking        true
rationale       Verifica que las flags no se activan espuriamente.
```

---

## Familia: circuitos con reloj (visión futura, niveles 11+)

Tareas cubiertas: latches, flip-flops, registros, memorias, datapaths. **No bloqueante todavía**; el catálogo de casos se prepara para Fase 13 del roadmap.

```
id              edge_clk_reset_returns_to_initial
family          temporal
applies_to      flip-flops, registros, memorias
inputs          secuencia: reset=1 durante 1 ciclo, luego reset=0,
                inputs cualesquiera durante N ciclos, reset=1 de nuevo
expected        estado al final = estado inicial después del segundo reset
blocking        true (cuando aplique)
rationale       Propiedad fundamental de reset.

id              edge_clk_enable_zero_holds_state
family          temporal
applies_to      flip-flops, registros con enable
inputs          enable = 0; inputs cambiantes
expected        estado no cambia mientras enable = 0
blocking        true (cuando aplique)

id              edge_clk_enable_one_propagates
family          temporal
applies_to      flip-flops, registros con enable
inputs          enable = 1; inputs definidos
expected        estado se actualiza en el flanco de reloj con el input
blocking        true (cuando aplique)

id              edge_clk_write_read_back
family          temporal
applies_to      registros, memorias
inputs          escribir un valor V en dirección D; ciclo después leer D
expected        valor leído = V
blocking        true (cuando aplique)

id              edge_clk_write_collision
family          temporal
applies_to      memorias con varios puertos
inputs          escrituras simultáneas a la misma dirección (si aplica)
expected        comportamiento determinista según política declarada
blocking        true cuando aplique; false si la tarea no soporta multipuerto

id              edge_clk_address_min_max
family          temporal
applies_to      memorias direccionables
inputs          escritura/lectura en dirección 0 y en dirección máxima
expected        correcto en ambos extremos
blocking        true (cuando aplique)

id              edge_clk_long_sequence_reproducible
family          temporal
applies_to      datapaths, registros, memorias
inputs          secuencia larga reproducible con semilla
expected        estado final coincide con modelo de referencia
blocking        true (cuando aplique)
```

---

## Política de inclusión y crecimiento

1. **Append-only.** Toda entrada que alguna vez detectó un bug se queda. Sin excepciones por "obsolescencia".
2. **Categorización obligatoria.** Cada caso pertenece a una familia y declara `applies_to` explícito.
3. **Versionado.** El catálogo lleva su propio `version`; cada incorporación incrementa el patch.
4. **Auditoría humana** requerida para eliminar o desbloquear una entrada (ver [25](25-human-audit-policy.md) §"Cambios de pruebas").
5. **No filtración.** Los casos concretos no se entregan al agente antes de evaluarse, salvo política de tarea explícita (ver [16](16-ai-visibility-limits.md)).

## Mapeo a suites

Una **suite** es un conjunto de casos límite agrupados para su uso por una tarea. Los nombres siguen la convención `edge_cases_<scope>_v<n>`:

| Suite | Casos incluidos (resumen) |
|------|---------------------------|
| `edge_cases_2input_v1` | `edge_all_zeros`, `edge_all_ones`, `edge_single_bit_on`, `edge_single_bit_off`, `edge_logic_complementary_pair`, `edge_logic_xor_corner`, `edge_logic_nand_corner` |
| `edge_cases_3input_v1` | universales + `edge_logic_majority_tie`, `edge_logic_majority_below`, `edge_alternating_pattern_*` |
| `edge_cases_mux_v1` | toda la familia mux |
| `edge_cases_cmp_v1` | toda la familia cmp |
| `edge_cases_add_v1` | familia adder, sin las de flags signed |
| `edge_cases_add_signed_v1` | añade `edge_add_signed_overflow_*` |
| `edge_cases_bus_v1` | familia bus_small |
| `edge_cases_flags_v1` | familia flags |
| `edge_cases_temporal_v1` | familia temporal (preparada para futuro) |

Las definiciones físicas concretas de estas suites se cierran cuando se implemente la memoria de pruebas (Fase 5 del roadmap). Sus **nombres y conjuntos lógicos** son normativos a partir de este documento.

## Garantías del catálogo

- **Cobertura mínima** de fallos clásicos en lógica digital documentados históricamente.
- **Estabilidad de identidad**: cada `id` es estable; las suites cambian añadiendo casos, no renumerándolos.
- **Independencia del agente**: los mismos casos se aplican a todos los agentes; no hay personalización oculta.
- **Trazabilidad**: cada caso registra `introduced_in` para entender la evolución del catálogo.

## Lo que el catálogo **no** hace

- **No reemplaza la suite exhaustiva** cuando ésta es viable. Los casos límite **complementan**, no sustituyen.
- **No favorece a un agente** entregándole los casos antes de la evaluación (ver [16](16-ai-visibility-limits.md), [18](18-operational-vs-non-operational-memory.md)).
- **No introduce primitivas.** Todos los casos se expresan como vectores de entrada y expectativas de salida; nada más.
- **No decide promoción.** Aporta evidencia al verificador; el verificador decide; el coordinador promueve.
