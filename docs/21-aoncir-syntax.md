# 21 — Sintaxis física de `.aoncir` (versión inicial, TOML)

> **Documento normativo.** Define la sintaxis física **propuesta para fase inicial** del formato `.aoncir`. El contenido lógico de un `.aoncir` (qué información representa) está fijado en [03 — Formato `.aoncir`](03-format-aoncir.md). Aquí se concreta **cómo se serializa**.
>
> **Decisión de fase inicial.** Se adopta **TOML legible** como sintaxis física. Las razones, las garantías y las alternativas futuras se discuten al final del documento. La elección no compromete a binario; un futuro `format_version` puede migrar a representación binaria o híbrida sin cambiar el modelo lógico.

## Principio reiterado

Independientemente de la sintaxis física, **un `.aoncir` solo contiene nodos de tipo AND, OR o NOT**. La sintaxis TOML descrita aquí **no admite** ningún otro `kind`. El parser estricto rechaza cualquier valor de `kind` fuera de `{"AND", "OR", "NOT"}`.

## Estructura física del archivo

Un archivo `.aoncir` en TOML inicial tiene exactamente las siguientes secciones, en este orden recomendado:

```
[format]            (obligatorio)
[meta]              (obligatorio)
[ports]             (obligatorio)
[[signals]]         (opcional, 0..*)
[[gates]]           (obligatorio, 1..*)
[[outputs]]         (obligatorio, 1..*)
[layout]            (opcional)
[verification]      (opcional, se rellena al verificar)
[metrics]           (opcional, se rellena al evaluar)
[history]           (opcional)
```

El parser estricto valida cada sección y cada campo. Campos desconocidos abortan la carga con error explícito.

---

## Sección `[format]`

Identifica la versión del formato físico.

```toml
[format]
format_version = "1.0.0"          # SemVer del formato físico
encoding       = "utf-8"
```

Reglas:

- `format_version` obligatorio. Si el parser no soporta esa versión, aborta con `L3 — incompatible format_version`.
- `encoding` debe ser `"utf-8"`. Otros valores son rechazados.

---

## Sección `[meta]`

Metadatos del circuito.

```toml
[meta]
name           = "one_bit_full_adder"          # nombre canónico (snake_case)
version        = "1.0.0"                       # versión del circuito (SemVer)
parameters     = { width = 1 }                 # parámetros estructurales (objeto)
level          = 5                             # nivel curricular (0..13 alcance inicial)
task_id        = "one_bit_full_adder"          # referencia a task del catálogo
hash_canonical = "blake3:..."                  # hash canónico (algoritmo:hex)
predecessor    = "blake3:..."                  # hash de versión anterior (o null)
author         = "agent_id_or_human"           # quién la produjo
created_at     = "2026-05-11T19:20:00Z"        # ISO-8601 UTC
status         = "official_active"             # ver tabla más abajo
```

`status` toma uno de los valores normados:

| Valor | Significado |
|------|-------------|
| `official_active` | Versión oficial activa en memoria canónica |
| `historical` | Versión previamente oficial, archivada |
| `experimental` | Verificada pero no promovida |
| `withdrawn` | Retirada por defecto detectado posteriormente |
| `imported` | Importación externa pendiente o verificada |
| `degraded` | Oficial activa que falló re-verificación con suite ampliada |

Cualquier otro valor invalida el archivo.

Reglas adicionales:

- `name` solo `[a-z][a-z0-9_]*`. No empieza con dígito. No contiene mayúsculas, espacios, guiones ni caracteres especiales.
- `name` no puede ser `"and_gate"`, `"or_gate"`, `"not_gate"`, `"xor_gate"`, `"xnor_gate"`, `"nand_gate"`, `"nor_gate"`, ni cualquier alias coloquial de primitiva. Esos nombres están **reservados** y el parser los rechaza.
- `parameters` es un objeto TOML inline; sus claves son nombres de parámetros estructurales (`width`, etc.) y sus valores son números enteros positivos o strings.
- `hash_canonical` se calcula sobre la topología abstracta y el campo `predecessor` (ver [03](03-format-aoncir.md) y [19](19-versioning-policy.md)). El algoritmo recomendado es **BLAKE3** prefijado con `blake3:`; cualquier algoritmo se identifica con prefijo.
- `predecessor` puede ser cadena vacía `""` o ausente si la versión es la primera.

---

## Sección `[ports]`

Puertos externos del circuito. Subdividida en `inputs` y `outputs`, **ambas declaradas como arrays de tablas** (`[[ports.inputs]]`, `[[ports.outputs]]`) para preservar el orden de aparición.

```toml
# Las entradas se declaran como un array de tablas.
# El ORDEN DE APARICIÓN ES NORMATIVO y define el orden del InputVector.

[[ports.inputs]]
name         = "operand_a"
semantic_tag = "operand_bit"
group        = "operand_a"

[[ports.inputs]]
name         = "operand_b"
semantic_tag = "operand_bit"
group        = "operand_b"

[[ports.inputs]]
name         = "carry_input"
semantic_tag = "carry"
group        = ""

# Las salidas se declaran de la misma forma.
# El ORDEN DE APARICIÓN ES NORMATIVO y define el orden del OutputVector.

[[ports.outputs]]
name         = "sum_output"
semantic_tag = "sum_bit"
group        = ""

[[ports.outputs]]
name         = "carry_output"
semantic_tag = "carry"
group        = ""
```

### Regla normativa P.1 — Orden de aparición = contrato formal del vector

El **orden en que aparecen** los bloques `[[ports.inputs]]` en el archivo define **el orden del `InputVector`** que recibe el simulador y el verificador. Idéntica regla para `[[ports.outputs]]` y el `OutputVector`. Esto es **contrato formal del circuito**: reordenar dos `[[ports.inputs]]` produce un circuito distinto desde el punto de vista de su interfaz, y por tanto un **hash canónico distinto**.

Consecuencia: el orden no se infiere de los nombres de los puertos (no hay orden lexicográfico implícito), no se infiere de la posición de las compuertas, y no se infiere del orden de declaración de las señales internas. **Solo cuenta el orden de aparición de `[[ports.inputs]]` y `[[ports.outputs]]`**.

### Regla normativa P.2 — Atributos de puerto

Cada `[[ports.inputs]]` y `[[ports.outputs]]` declara:

| Campo | Obligatorio | Significado |
|------|-------------|-------------|
| `name` | sí | Identificador snake_case único en el archivo (no colisiona con otros puertos, señales ni gates). |
| `semantic_tag` | sí | Etiqueta del catálogo de [24 — Convenciones de etiquetas semánticas](24-semantic-tag-conventions.md). Puede ser cadena vacía `""` si el rol es genérico. |
| `group` | sí | Cadena vacía `""` si el puerto no pertenece a ningún grupo; nombre del grupo si pertenece a uno declarado en `[[semantic_groups]]`. |
| `bit_position` | no | Entero ≥ 0. Significativo solo dentro de un mismo `group`; define el orden de bits del bus. No se usa en circuitos de Fase 1 (puertos sueltos). Documentado para uso futuro en buses multi-bit. |

AONIX trabaja bit a bit: cada puerto representa **un único bit**. Los buses se modelan como conjuntos de puertos individuales que comparten `group` y se distinguen por `bit_position`. No existen puertos "anchos" como tipo primitivo.

### Regla normativa P.3 — Cardinalidad

- `[[ports.inputs]]` puede estar vacío (raro; ejemplo: circuitos constantes con todas las entradas conceptualmente ignorables — para Fase 1, los circuitos canónicos siempre tienen al menos una entrada).
- `[[ports.outputs]]` debe tener al menos **1** entrada. Sin salidas, el circuito no tiene función observable.

### Regla normativa P.4 — Bit position dentro de un grupo

Cuando varios puertos pertenecen al mismo `group`:

- `bit_position` debe ser **único** dentro del grupo.
- `bit_position` debe formar un **rango contiguo desde 0** hasta `width - 1` (siendo `width` el número de miembros del grupo).
- La convención canónica de mapeo `bit_position` ↔ MSB/LSB queda fijada por AONIX: **`bit_position = 0` es siempre LSB**, `bit_position = width - 1` es siempre MSB. Esta regla es normativa, fija y sin parámetro de instalación (ver [24 — Convenciones de etiquetas semánticas §U.7](24-semantic-tag-conventions.md)).

Para puertos **sin grupo** (puertos sueltos), `bit_position` es irrelevante; si está, se ignora.

### Regla normativa P.5 — Validación del parser

El parser estricto rechaza:

1. `name` duplicado entre cualquier par de puertos, señales o gates del archivo.
2. `semantic_tag` no perteneciente al catálogo (modo estricto).
3. `group` referenciado en un puerto sin declaración en `[[semantic_groups]]`.
4. `bit_position` duplicado dentro de un mismo `group`.
5. `bit_position` no contiguo dentro de un mismo `group`.
6. `[[ports.outputs]]` vacío.
7. Cualquier campo desconocido en un `[[ports.*]]`.

### Grupos semánticos (opcional)

Cuando varios puertos pertenecen a un mismo bus o flag-set:

```toml
[[semantic_groups]]
id      = "operand_a"
kind    = "bus"
members = ["a"]               # puertos por nombre
width   = 1

[[semantic_groups]]
id      = "alu_flags"
kind    = "flags"
members = ["zero_flag", "carry_flag", "overflow_flag", "negative_flag"]
```

`kind` toma uno de los valores normados de [24](24-semantic-tag-conventions.md): `bus`, `address_bus`, `data_bus`, `flags`, `select_bus`, `control_bus`, `operand`. Otros valores rechazados.

---

## Sección `[[signals]]`

Señales **internas** (no externas). Las señales de entrada y salida ya están declaradas en `[ports]`; aquí se declaran las que viven dentro.

```toml
[[signals]]
id           = "s1"
semantic_tag = ""            # opcional, vacío si no aplica
group        = ""

[[signals]]
id           = "s2"
semantic_tag = ""

[[signals]]
id           = "not_a"
semantic_tag = ""
```

Reglas:

- `id` snake_case único en el archivo (no colisiona con nombres de puertos).
- Una señal interna sin uso (no entra a ningún `[[gates]]` ni se asigna a ningún `[[outputs]]`) es **señal muerta**. Detectada por el parser para reporte; tolerada en versiones experimentales, **rechazada en `status = "official_active"`**.

---

## Sección `[[gates]]`

Nodos lógicos. **Una compuerta por entrada de la lista.**

```toml
[[gates]]
id     = "g1"
kind   = "NOT"
inputs = ["b"]
output = "not_b"

[[gates]]
id     = "g2"
kind   = "AND"
inputs = ["a", "not_b"]
output = "s1"

[[gates]]
id     = "g3"
kind   = "NOT"
inputs = ["a"]
output = "not_a"

[[gates]]
id     = "g4"
kind   = "AND"
inputs = ["not_a", "b"]
output = "s2"

[[gates]]
id     = "g5"
kind   = "OR"
inputs = ["s1", "s2"]
output = "a_xor_b"             # comportamiento XOR — nombre libre del agente

# ... resto del grafo
```

Reglas de validación del parser:

1. `kind` ∈ `{"AND", "OR", "NOT"}` exactamente. **Cualquier otro valor invalida el archivo (L3, causa: ABSOLUTE_RULE).**
2. Si `kind = "NOT"` ⇒ `inputs` tiene **exactamente 1** elemento.
3. Si `kind = "AND"` o `"OR"` ⇒ `inputs` tiene **exactamente 2** elementos (aridad estricta de Fase 1; ver [01 — Reglas absolutas](01-rules-absolute.md)).
4. Cada elemento de `inputs` referencia el nombre de un puerto de entrada o de una señal interna. **Las constantes (`"0"`, `"1"`) no son referencias válidas en Fase 1**; un circuito que necesite un valor constante debe derivarlo (por ejemplo, `y = a AND NOT a` ≡ 0).
5. `output` referencia una señal interna declarada en `[[signals]]` o (excepcionalmente) un puerto de salida si la compuerta alimenta directamente la salida (en cuyo caso `[[outputs]]` puede declararse implícitamente; ver siguiente sección).
6. `id` de compuerta es único en el archivo.
7. El grafo derivado debe ser DAG salvo en niveles ≥ 11 donde feedback temporal con gating por `clock` se permite explícitamente.
8. No puede aparecer un `id` de compuerta con sintaxis `xor*`, `nand*`, `nor*`, `xnor*` en modo estricto, para evitar confusión semántica. (Los nombres libres del agente para señales internas no están restringidos, pero los IDs de compuerta y los puertos sí siguen esta política.)

---

## Sección `[[outputs]]`

Asignaciones de puertos de salida a señales (puertos o señales internas).

```toml
[[outputs]]
port   = "sum"
source = "a_xor_b_xor_cin"    # señal interna que produce el bit de suma

[[outputs]]
port   = "cout"
source = "carry_out_final"
```

Reglas:

- Cada puerto declarado en `[[ports.outputs]]` aparece exactamente **una vez** como `port` en `[[outputs]]`.
- `source` referencia una señal interna o un puerto de entrada (raro; ej. `pass_through`).
- No se permiten constantes (`"0"`, `"1"`) como `source` ni como elemento de `[[gates]].inputs` en Fase 1. Una versión oficial activa nunca contiene referencias a constantes primitivas. Si se necesita un valor constante, el circuito lo deriva con compuertas (`y = a AND NOT a` ≡ 0, `y = a OR NOT a` ≡ 1).

---

## Sección `[layout]` (opcional)

Posiciones 2D para visualización. No afecta el hash canónico.

```toml
[layout]
strategy = "layered"           # layered | force_directed | manual | bus_aware

[[layout.positions]]
target = "gate:g1"             # gate:<id> | signal:<id> | port:<name>
x      = 12
y      = 4

[[layout.positions]]
target = "gate:g2"
x      = 12
y      = 6

# ...
```

Reglas:

- Coordenadas son números (enteros o reales).
- `target` solo referencia elementos existentes.
- Si `strategy = "force_directed"`, debe acompañar `seed = <u64>` para determinismo (ver [09](09-visualization-vulkan.md)).

---

## Sección `[verification]` (rellenada por el verificador)

Sello de verificación cuando se sirve la versión oficial activa.

```toml
[verification]
verified_at      = "2026-05-11T19:25:00Z"
verifier_version = "1.0.0"
suites = [
    { id = "exhaustive_suite_8cases",    version = "1.0.0", passed = 8,    total = 8,    seed = "" },
    { id = "edge_cases_3input_v1",       version = "1.0.0", passed = 12,   total = 12,   seed = "" },
    { id = "regression_suite_adder_family_v1", version = "1.2.0", passed = 47, total = 47, seed = "0xCAFEBABE" },
]
reference_models = [
    { id = "full_adder_1bit_software",   version = "1.0.0", diff = "" },
]
result = "PASS"
```

Reglas:

- `result` ∈ `{"PASS", "FAIL"}`. No hay valores intermedios.
- Si `result = "FAIL"`, el archivo no puede tener `status = "official_active"`.

---

## Sección `[metrics]` (rellenada por el evaluador)

Métricas estructurales en el momento de la verificación.

```toml
[metrics]
evaluator_version = "1.0.0"
gate_count        = { AND = 5, OR = 2, NOT = 2, TOTAL = 9 }
depth             = 4
signals_internal  = 8
dead_signals      = 0
fan_in_max        = 3
fan_out_max       = 3
reuse_score       = 0.42
critical_path     = ["a", "g2", "s1", "g5", "a_xor_b", "g8", "..."]
visual_complexity_estimate = 0.31
```

---

## Sección `[history]` (opcional)

Cadena histórica acumulada (útil para inspección humana; redundante con el campo `predecessor` de `[meta]`, pero más cómoda de leer).

```toml
[history]
ancestors = [
    "blake3:abcd...1234",
    "blake3:efgh...5678",
]
```

---

## Ejemplo completo 1 — `multiplexer_2_to_1.aoncir`

Nombres explícitos en todo el archivo. Sin abreviaturas como `d0`, `d1`, `s`, `y`.

```toml
[format]
format_version = "1.0.0"
encoding       = "utf-8"

[meta]
name           = "multiplexer_2_to_1"
version        = "1.0.0"
parameters     = {}
level          = 5
task_id        = "multiplexer_2_to_1"
hash_canonical = "blake3:placeholder_hash_will_be_computed"
predecessor    = ""
author         = "search_exhaustive_v1"
created_at     = "2026-05-11T19:20:00Z"
status         = "official_active"

# Orden de aparición = contrato formal del InputVector:
# [ data_input_zero, data_input_one, select_input ]

[[ports.inputs]]
name         = "data_input_zero"
semantic_tag = "data_bit"
group        = ""

[[ports.inputs]]
name         = "data_input_one"
semantic_tag = "data_bit"
group        = ""

[[ports.inputs]]
name         = "select_input"
semantic_tag = "select"
group        = ""

# Orden de aparición = contrato formal del OutputVector:
# [ data_output ]

[[ports.outputs]]
name         = "data_output"
semantic_tag = "data_bit"
group        = ""

[[signals]]
id = "select_input_negated"
[[signals]]
id = "gated_data_zero"
[[signals]]
id = "gated_data_one"
[[signals]]
id = "data_output_internal"

[[gates]]
id     = "g1"
kind   = "NOT"
inputs = ["select_input"]
output = "select_input_negated"

[[gates]]
id     = "g2"
kind   = "AND"
inputs = ["select_input_negated", "data_input_zero"]
output = "gated_data_zero"

[[gates]]
id     = "g3"
kind   = "AND"
inputs = ["select_input", "data_input_one"]
output = "gated_data_one"

[[gates]]
id     = "g4"
kind   = "OR"
inputs = ["gated_data_zero", "gated_data_one"]
output = "data_output_internal"

[[outputs]]
port   = "data_output"
source = "data_output_internal"

[verification]
verified_at      = "2026-05-11T19:25:00Z"
verifier_version = "1.0.0"
suites = [
    { id = "exhaustive_suite_8cases", version = "1.0.0", passed = 8, total = 8, seed = "" },
    { id = "property_mux_select_behavior", version = "1.0.0", passed = 1, total = 1, seed = "" },
    { id = "regression_suite_mux_family_v1", version = "1.0.0", passed = 5, total = 5, seed = "0xC0DE" },
]
result = "PASS"

[metrics]
evaluator_version = "1.0.0"
gate_count        = { AND = 2, OR = 1, NOT = 1, TOTAL = 4 }
depth             = 3
signals_internal  = 4
dead_signals      = 0
fan_in_max        = 2
fan_out_max       = 1
reuse_score       = 0.0
```

---

## Ejemplo completo 2 — `one_bit_full_adder.aoncir`

Nombres explícitos: `operand_a`, `operand_b`, `carry_input`, `sum_output`, `carry_output`. Las señales internas que implementan el comportamiento XOR aparecen como `exclusive_or_behavior_*` para dejar claro que son **composición de AND/OR/NOT**, no primitivas.

```toml
[format]
format_version = "1.0.0"
encoding       = "utf-8"

[meta]
name           = "one_bit_full_adder"
version        = "1.0.0"
parameters     = { width = 1 }
level          = 5
task_id        = "one_bit_full_adder"
hash_canonical = "blake3:placeholder_hash_will_be_computed"
predecessor    = ""
author         = "search_exhaustive_v1"
created_at     = "2026-05-11T19:21:00Z"
status         = "official_active"

# Orden de aparición = contrato formal del InputVector:
# [ operand_a, operand_b, carry_input ]

[[ports.inputs]]
name         = "operand_a"
semantic_tag = "operand_bit"
group        = "operand_a_group"

[[ports.inputs]]
name         = "operand_b"
semantic_tag = "operand_bit"
group        = "operand_b_group"

[[ports.inputs]]
name         = "carry_input"
semantic_tag = "carry"
group        = ""

# Orden de aparición = contrato formal del OutputVector:
# [ sum_output, carry_output ]

[[ports.outputs]]
name         = "sum_output"
semantic_tag = "sum_bit"
group        = ""

[[ports.outputs]]
name         = "carry_output"
semantic_tag = "carry"
group        = ""

[[semantic_groups]]
id      = "operand_a_group"
kind    = "operand"
members = ["operand_a"]
width   = 1

[[semantic_groups]]
id      = "operand_b_group"
kind    = "operand"
members = ["operand_b"]
width   = 1

[[signals]]
id = "operand_a_negated"
[[signals]]
id = "operand_b_negated"
[[signals]]
id = "operand_a_and_operand_b_negated"
[[signals]]
id = "operand_a_negated_and_operand_b"
[[signals]]
id = "exclusive_or_behavior_of_operands"          # (a AND NOT b) OR (NOT a AND b)
[[signals]]
id = "carry_input_negated"
[[signals]]
id = "exclusive_or_and_carry_input_negated"
[[signals]]
id = "exclusive_or_negated"
[[signals]]
id = "exclusive_or_negated_and_carry_input"
[[signals]]
id = "sum_output_internal"
[[signals]]
id = "operand_a_and_operand_b"
[[signals]]
id = "exclusive_or_and_carry_input"
[[signals]]
id = "carry_output_internal"

# operand_a_negated = NOT operand_a
[[gates]]
id     = "g1"
kind   = "NOT"
inputs = ["operand_a"]
output = "operand_a_negated"

# operand_b_negated = NOT operand_b
[[gates]]
id     = "g2"
kind   = "NOT"
inputs = ["operand_b"]
output = "operand_b_negated"

# operand_a AND NOT operand_b
[[gates]]
id     = "g3"
kind   = "AND"
inputs = ["operand_a", "operand_b_negated"]
output = "operand_a_and_operand_b_negated"

# NOT operand_a AND operand_b
[[gates]]
id     = "g4"
kind   = "AND"
inputs = ["operand_a_negated", "operand_b"]
output = "operand_a_negated_and_operand_b"

# (operand_a AND NOT operand_b) OR (NOT operand_a AND operand_b)  -- comportamiento XOR
[[gates]]
id     = "g5"
kind   = "OR"
inputs = ["operand_a_and_operand_b_negated", "operand_a_negated_and_operand_b"]
output = "exclusive_or_behavior_of_operands"

# carry_input_negated = NOT carry_input
[[gates]]
id     = "g6"
kind   = "NOT"
inputs = ["carry_input"]
output = "carry_input_negated"

# exclusive_or_behavior_of_operands AND NOT carry_input
[[gates]]
id     = "g7"
kind   = "AND"
inputs = ["exclusive_or_behavior_of_operands", "carry_input_negated"]
output = "exclusive_or_and_carry_input_negated"

# exclusive_or_negated = NOT exclusive_or_behavior_of_operands
[[gates]]
id     = "g8"
kind   = "NOT"
inputs = ["exclusive_or_behavior_of_operands"]
output = "exclusive_or_negated"

# exclusive_or_negated AND carry_input
[[gates]]
id     = "g9"
kind   = "AND"
inputs = ["exclusive_or_negated", "carry_input"]
output = "exclusive_or_negated_and_carry_input"

# sum_output_internal = (exclusive_or AND NOT carry_input) OR (NOT exclusive_or AND carry_input)
# Es decir: comportamiento XOR triple sobre operand_a, operand_b, carry_input
[[gates]]
id     = "g10"
kind   = "OR"
inputs = ["exclusive_or_and_carry_input_negated", "exclusive_or_negated_and_carry_input"]
output = "sum_output_internal"

# carry parcial 1: operand_a AND operand_b
[[gates]]
id     = "g11"
kind   = "AND"
inputs = ["operand_a", "operand_b"]
output = "operand_a_and_operand_b"

# carry parcial 2: exclusive_or_behavior_of_operands AND carry_input
[[gates]]
id     = "g12"
kind   = "AND"
inputs = ["exclusive_or_behavior_of_operands", "carry_input"]
output = "exclusive_or_and_carry_input"

# carry_output_internal = operand_a_and_operand_b OR exclusive_or_and_carry_input
[[gates]]
id     = "g13"
kind   = "OR"
inputs = ["operand_a_and_operand_b", "exclusive_or_and_carry_input"]
output = "carry_output_internal"

[[outputs]]
port   = "sum_output"
source = "sum_output_internal"

[[outputs]]
port   = "carry_output"
source = "carry_output_internal"

[verification]
verified_at      = "2026-05-11T19:25:00Z"
verifier_version = "1.0.0"
suites = [
    { id = "exhaustive_suite_8cases",     version = "1.0.0", passed = 8,  total = 8,  seed = "" },
    { id = "edge_cases_3input_v1",        version = "1.0.0", passed = 12, total = 12, seed = "" },
    { id = "property_full_adder_sum_arithmetic",   version = "1.0.0", passed = 1, total = 1, seed = "" },
    { id = "property_full_adder_carry_arithmetic", version = "1.0.0", passed = 1, total = 1, seed = "" },
    { id = "regression_suite_adder_family_v1",     version = "1.0.0", passed = 12, total = 12, seed = "0xADDE" },
    { id = "semantic_signal_check.carry",          version = "1.0.0", passed = 1, total = 1, seed = "" },
]
reference_models = [
    { id = "full_adder_1bit_software", version = "1.0.0", diff = "" },
]
result = "PASS"

[metrics]
evaluator_version = "1.0.0"
gate_count        = { AND = 6, OR = 3, NOT = 4, TOTAL = 13 }
depth             = 5
signals_internal  = 13
dead_signals      = 0
fan_in_max        = 2
fan_out_max       = 2
reuse_score       = 0.31
visual_complexity_estimate = 0.28
```

---

## Ejemplo completo 3 — `two_bit_full_adder.aoncir` (esbozo)

Por brevedad solo se muestra la estructura de puertos y grupos; la expansión interna completa repite el patrón de `one_bit_full_adder` dos veces, encadenando `carry_output_bit_zero` al `carry_input_bit_one`. Aquí aparece el primer uso real de `bit_position` para fijar el orden de bits dentro de un bus.

```toml
[format]
format_version = "1.0.0"

[meta]
name           = "two_bit_full_adder"
version        = "1.0.0"
parameters     = { width = 2 }
level          = 8
task_id        = "two_bit_full_adder"
hash_canonical = "blake3:placeholder"
predecessor    = ""
status         = "official_active"
# ... resto del meta

# Orden de aparición de [[ports.inputs]] = contrato del InputVector:
# [ operand_a_bit_zero, operand_a_bit_one,
#   operand_b_bit_zero, operand_b_bit_one,
#   carry_input ]

[[ports.inputs]]
name         = "operand_a_bit_zero"
semantic_tag = "operand_bit"
group        = "operand_a_bus"
bit_position = 0

[[ports.inputs]]
name         = "operand_a_bit_one"
semantic_tag = "operand_bit"
group        = "operand_a_bus"
bit_position = 1

[[ports.inputs]]
name         = "operand_b_bit_zero"
semantic_tag = "operand_bit"
group        = "operand_b_bus"
bit_position = 0

[[ports.inputs]]
name         = "operand_b_bit_one"
semantic_tag = "operand_bit"
group        = "operand_b_bus"
bit_position = 1

[[ports.inputs]]
name         = "carry_input"
semantic_tag = "carry"
group        = ""

# Orden de aparición de [[ports.outputs]] = contrato del OutputVector:
# [ sum_output_bit_zero, sum_output_bit_one, carry_output ]

[[ports.outputs]]
name         = "sum_output_bit_zero"
semantic_tag = "sum_bit"
group        = "sum_output_bus"
bit_position = 0

[[ports.outputs]]
name         = "sum_output_bit_one"
semantic_tag = "sum_bit"
group        = "sum_output_bus"
bit_position = 1

[[ports.outputs]]
name         = "carry_output"
semantic_tag = "carry"
group        = ""

[[semantic_groups]]
id      = "operand_a_bus"
kind    = "bus"
members = ["operand_a_bit_zero", "operand_a_bit_one"]
width   = 2

[[semantic_groups]]
id      = "operand_b_bus"
kind    = "bus"
members = ["operand_b_bit_zero", "operand_b_bit_one"]
width   = 2

[[semantic_groups]]
id      = "sum_output_bus"
kind    = "bus"
members = ["sum_output_bit_zero", "sum_output_bit_one"]
width   = 2

# Aquí siguen las señales internas y las compuertas expandidas a AND/OR/NOT
# para el ripple-carry adder de 2 bits. Cada bit replica la lógica del
# one_bit_full_adder con sus señales propias.
# Importante: NINGUNA compuerta xor / nand / nor / xnor aparece; todo
# se construye con AND, OR y NOT.
```

La versión oficial activa serializa el grafo **completamente expandido**, sin invocaciones jerárquicas opacas. Si el agente piensa en términos de "dos full adders encadenados", esa es su organización mental; el `.aoncir` final contiene todas las compuertas primitivas.

---

## Convenciones léxicas

| Concepto | Regla |
|---------|-------|
| Identificadores de circuito | snake_case, empieza por letra, `[a-z][a-z0-9_]*` |
| Identificadores de compuerta | `g<n>` recomendado, libre snake_case |
| Identificadores de señal | snake_case, libre, evitar prefijo `xor`, `nand`, `nor`, `xnor` para no inducir confusión con la lista negra |
| Nombres de puerto | snake_case |
| Etiquetas semánticas | snake_case, según catálogo de [24](24-semantic-tag-conventions.md) |
| Indentación | 2 espacios (TOML estándar) |
| Comentarios | `#` (TOML estándar), permitidos en todo el archivo |
| Cadenas | UTF-8 con escape estándar TOML |
| Booleanos | `true` / `false` |
| Enteros | decimales |
| Reales | TOML estándar |

### Regla de nomenclatura explícita

Todos los identificadores en un `.aoncir` —puertos, señales, compuertas, grupos, etiquetas— deben ser **explícitos en inglés y sin abreviaturas innecesarias**. Esta regla aplica al contenido de los `.aoncir` producidos por AONIX y a los ejemplos canónicos de la documentación.

Sustituciones obligatorias por convención:

| Abreviatura prohibida | Forma explícita recomendada |
|----------------------|------------------------------|
| `cin`, `ci` | `carry_input` |
| `cout`, `co` | `carry_output` |
| `d0`, `d1`, `d2`, `d3` | `data_input_zero`, `data_input_one`, `data_input_two`, `data_input_three` |
| `s`, `sel` | `select_input` |
| `y`, `out` (genéricos) | `data_output` o nombre más específico del rol |
| `a`, `b` (cuando son operandos aritméticos) | `operand_a`, `operand_b` |
| `id` (como sufijo conceptual) | `identifier` (en código fuente Rust). En claves TOML como `id` se conserva por convención del propio formato. |
| `sim`, `eval`, `topo` | no aplican a contenido del `.aoncir`, pero aplican a código y nombres de archivo de tests. |

El parser estricto **no rechaza** identificadores abreviados como falla técnica (el `.aoncir` se carga sin error), pero el productor de `.aoncir` (humano, búsqueda, IA) debe respetar la convención. AONIX puede emitir warning informativo cuando detecte abreviaturas comunes.

Esta regla forma parte del contrato de calidad documental, no de R1/R2. Su incumplimiento no invalida un circuito como verdad técnica, pero degrada legibilidad, audit y aprendizaje.

## Validaciones del parser estricto (lista no exhaustiva)

```
PARSER STRICT VALIDATIONS

1.  Archivo es UTF-8 sin BOM excepto BOM permitido como tolerancia explícita.
2.  Sintaxis TOML válida; cualquier error sintáctico aborta.
3.  Secciones obligatorias presentes: [format], [meta], [ports], al menos
    1 [[gates]], al menos 1 [[outputs]].
4.  [format].format_version está dentro del rango soportado por el parser.
5.  [meta].name conforma a regex snake_case y no es nombre reservado.
6.  [meta].name no contiene "xor", "nand", "nor", "xnor" como token aislado
    al inicio salvo en composiciones legítimas como
    "xor_behavior_demonstrator" — política configurable. (Modo estricto
    inicial: rechazar prefijo coincidente con derivada para forzar
    naming descriptivo.)
7.  [meta].status ∈ conjunto normado.
8.  Todo gate.kind ∈ {"AND", "OR", "NOT"}.
9.  Aridades respetadas según kind.
10. Todo input de gate referencia entidad existente (port input, signal,
    sin constantes en Fase 1).
11. Toda signal declarada se referencia al menos una vez (modo estricto
    para status="official_active"; warning para experimental).
12. Cada [[outputs]].port aparece en [[ports.outputs]] y exactamente una
    vez en la lista de outputs.
13. Cada [[outputs]].source referencia una entidad existente.
14. Sin ciclos no permitidos (modo combinacional).
15. Hash canónico recalculado coincide con [meta].hash_canonical.
16. Predecesor (si existe) está accesible en memoria histórica (modo
    estricto online; modo offline tolera huérfanos con warning).
17. Etiquetas semánticas en [[ports.inputs]], [[ports.outputs]], [[signals]],
    [[semantic_groups]] pertenecen al catálogo de [24].
18. El orden de aparición de [[ports.inputs]] y [[ports.outputs]] se
    preserva tras parsear; el parser NO los reordena (regla P.1).
19. Para puertos con `group` no vacío, `bit_position` es entero >= 0,
    único dentro del grupo, y forma rango contiguo desde 0 hasta
    width - 1 del grupo (regla P.4).
20. Para puertos sin `group`, `bit_position` (si está presente) se
    ignora; el parser puede emitir warning informativo.
```

## Garantías del formato físico

1. **Legible humanamente.** Un revisor puede auditar a mano un `.aoncir` pequeño.
2. **Diffable.** Cambios entre versiones aparecen claros en `git diff`.
3. **Parser estricto.** No hay modo permisivo en producción; los errores se reportan con localización.
4. **Determinismo del hash canónico.** El hash depende **solo** del modelo lógico (topología, puertos, etiquetas) y `predecessor`; **no** depende de orden de aparición de `[[gates]]` ni de comentarios ni de espacios.
5. **Compatibilidad hacia adelante.** Versiones nuevas del formato deben soportar carga de versiones anteriores hasta donde sea razonable.
6. **Independencia del backend de almacenamiento.** El TOML es archivo plano; el índice/DB es ortogonal.

## Lo que el formato físico **no** hace

- **No introduce primitivas.** El parser estricto rechaza todo `kind` fuera de AND/OR/NOT.
- **No es la única opción futura.** Una versión binaria o híbrida puede incorporarse en una `format_version` superior sin invalidar archivos existentes.
- **No fuerza orden** de las secciones más allá del orden recomendado; el parser tolera cualquier orden válido en TOML.
- **No es responsable de la decisión "promover/no promover".** El formato **describe**; la decisión la toma el coordinador.

## Alternativas futuras (no fase inicial)

| Alternativa | Razón a futuro | Coste de migración |
|------------|---------------|---------------------|
| Binario propio (similar a Cap'n Proto o postcard) | Velocidad, tamaño, hashing trivial | Necesita herramientas para inspección humana |
| Híbrido binario+textual | Mejor de ambos mundos | Doble path de parser |
| JSON Schema versionado | Compatibilidad ecosistema | Menos legible que TOML para árboles profundos |
| S-expressions | Tradición de circuit description | Curva de adopción humana |
| DSL propio | Expresividad máxima | Implementación significativa |

Todas estas alternativas son legítimas; ninguna se compromete a corto plazo. **La migración entre versiones físicas no invalida los `.aoncir` ya producidos**: AONIX mantiene parsers retrocompatibles.

## Pruebas mínimas del parser (a implementar en Fase 1, no ahora)

- Carga exitosa de los tres ejemplos completos de este documento.
- Rechazo correcto de `kind = "XOR"` con mensaje específico.
- Rechazo correcto de aridad incorrecta (`NOT` con 2 inputs, `AND` con 1 input).
- Rechazo correcto de señal indefinida.
- Rechazo correcto de ciclo.
- Rechazo correcto de salida sin asignar.
- Rechazo correcto de duplicado de id.
- Rechazo correcto de etiqueta semántica desconocida (modo estricto).
- Cálculo correcto del hash canónico (invariante a orden de secciones).
- Round-trip: parse → serialize → parse produce el mismo modelo lógico.

## Decisión cerrada (Fase 0 → Fase 1)

- **Formato físico inicial:** TOML legible 1.0.0.
- **Hash canónico:** BLAKE3 (recomendado, decisión final a confirmar; el formato físico permite cualquier algoritmo prefijado).
- **Encoding:** UTF-8.
- **Strict mode:** activo por defecto en producción.

## Decisiones que siguen abiertas

- Algoritmo final de hash canónico (BLAKE3 vs SHA-256 vs otro).
- Política exacta de tolerancia para nombres de circuito con prefijo coincidente con compuerta derivada (rechazo absoluto vs warning).
- Mecanismo de extensión para etiquetas semánticas custom de instalaciones específicas (lista cerrada vs lista cerrada + namespace).
- Algoritmo de canonicalización del orden topológico para el cálculo del hash.
