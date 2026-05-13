# 24 — Convenciones de etiquetas semánticas

> **Documento normativo.** Cataloga las etiquetas semánticas iniciales reconocidas por AONIX, su significado, reglas de uso, conflictos entre etiquetas, y políticas de agrupación (buses, flags, control). El parser estricto de `.aoncir` rechaza etiquetas no listadas en este catálogo salvo configuración explícita de extensión namespaced.
>
> **Principio reiterado:** una etiqueta semántica **no es** una operación, **no es** una compuerta, **no introduce primitivas**. Solo informa al simulador, verificador, evaluador, visualizador y traductor sobre **qué papel** juega una señal en el circuito. AONIX puede aprovechar la información para verificación por señal semántica, layout, explicaciones humanas y traducción para IA. La verdad estructural del circuito permanece intacta.

## Forma de las etiquetas

- snake_case, `[a-z][a-z0-9_]*`.
- Sin sufijos numéricos para distinguir bits dentro de un mismo bus (la pertenencia al bus se declara con `group`, no con el nombre).
- Caso-sensible. `carry` y `Carry` son distintos; el parser estricto solo acepta minúsculas.

## Catálogo de etiquetas semánticas iniciales

### Categoría 1 — Datos genéricos

| Etiqueta | Aplica a | Significado | Restricciones |
|---------|---------|-------------|---------------|
| `data_bit` | señal o puerto | Bit de datos sin semántica especializada | Ninguna |
| `operand_bit` | señal o puerto | Bit que forma parte de un operando aritmético | Suele pertenecer a un `group` de tipo `operand` |
| `sum_bit` | señal o puerto | Bit de resultado de una operación de suma | Producido típicamente como salida de un sumador |
| `difference_bit` | señal o puerto | Bit de resultado de una operación de resta | Producido por restadores |
| `parity_bit` | señal o puerto | Bit que representa una paridad calculada | Salida de XOR de N bits como comportamiento |
| `pattern_match` | señal o puerto | Bit alto cuando se detecta un patrón específico | Niveles 3+ |

### Categoría 2 — Acarreo y signo

| Etiqueta | Aplica a | Significado | Restricciones |
|---------|---------|-------------|---------------|
| `carry` | señal o puerto | Acarreo entre bits o entre operaciones | Verificación por señal semántica obligatoria |
| `borrow` | señal o puerto | Préstamo (acarreo de resta) | Idem; usado en restadores |
| `sign_bit` | señal o puerto | Bit de signo de un valor signed (MSB) | Convención: 0 = positivo, 1 = negativo (complemento a 2) |

### Categoría 3 — Flags aritméticas y de control

Las flags se agrupan en `semantic_groups` de tipo `flags` cuando varias aparecen juntas (típicamente en ALUs).

| Etiqueta | Aplica a | Significado | Restricciones |
|---------|---------|-------------|---------------|
| `zero_flag` | señal o puerto | Alta si el resultado de la operación es cero | Verificación semántica obligatoria desde nivel 9 |
| `carry_flag` | señal o puerto | Acarreo final unsigned | Idem |
| `overflow_flag` | señal o puerto | Overflow signed detectado | Idem |
| `negative_flag` | señal o puerto | Resultado signed negativo (= sign_bit del resultado) | Idem |
| `parity_flag` | señal o puerto | Paridad del resultado | Opcional |
| `equal_flag` | señal o puerto | Resultado de comparación: igualdad | Suele acompañarse de greater_flag y less_flag |
| `greater_flag` | señal o puerto | Resultado de comparación: mayor | |
| `less_flag` | señal o puerto | Resultado de comparación: menor | |

Cualquier circuito que declare salidas con etiquetas de flags **debe** declarar un `semantic_group` `kind = "flags"` que las agrupe, salvo casos triviales con una sola flag.

### Categoría 4 — Selectores y multiplexación

| Etiqueta | Aplica a | Significado | Restricciones |
|---------|---------|-------------|---------------|
| `select` | señal o puerto | Bit de selección en un mux/demux | Verificación semántica recomendada |
| `select_output` | señal o puerto | Salida de un decoder one-hot | Forma típicamente parte de un grupo `bus` de N salidas |
| `comparison` | señal o puerto | Salida de un comparador (eq, gt, lt) | Verificación por exclusión mutua recomendada |

### Categoría 5 — Temporalidad (niveles ≥ 11)

| Etiqueta | Aplica a | Significado | Restricciones |
|---------|---------|-------------|---------------|
| `clock` | puerto (típicamente input) | Señal de reloj | **No es una primitiva.** Activa el modo temporal del simulador. Una sola señal `clock` por dominio de reloj. |
| `reset` | puerto | Señal de reset asíncrono o síncrono según convención | Compatible con `clock` cuando aplique |
| `enable` | puerto | Habilita actualización de estado | Compatible con `clock` |
| `write_enable` | puerto | Habilita escritura (memorias, registros) | Compatible con `enable` |
| `read_enable` | puerto | Habilita lectura | Compatible con `enable` |
| `clock_edge_marker` | señal interna (uso del simulador) | Marca interna; **no permitida en `.aoncir` de usuario** | Reservada para uso del simulador en modo temporal |

### Categoría 6 — Buses y agrupaciones

Las etiquetas de bus se aplican como `kind` del `semantic_group`, no como etiqueta de señal individual; los bits del bus llevan etiquetas individuales de Categoría 1 o 2.

| `kind` del grupo | Significado | Restricciones |
|------------------|-------------|---------------|
| `bus` | Bus genérico de N bits | width declarado |
| `address_bus` | Bus de direcciones (memoria) | width declarado; orden LSB-first canónico (ver §U.7) |
| `data_bus` | Bus de datos | Idem |
| `select_bus` | Bus de selección (multi-bit select para muxes) | Idem |
| `control_bus` | Bus de señales de control agrupadas | Width arbitraria |
| `operand` | Operando aritmético multibit | Suele pertenecer al input de un sumador/restador |
| `flags` | Conjunto de flags semánticas | Sus miembros llevan etiquetas individuales de Categoría 3 |

### Categoría 7 — Control genérico

| Etiqueta | Aplica a | Significado | Restricciones |
|---------|---------|-------------|---------------|
| `control_signal` | señal o puerto | Señal de control no especificada con mayor precisión | Etiqueta de fallback; preferir etiquetas más específicas cuando aplican |

### Categoría 8 — Niveles arquitectónicos (visión futura, niveles 13+)

Reservadas para uso en CPU mínima y bloques arquitectónicos avanzados. Listadas para preservar coherencia futura.

| Etiqueta | Significado |
|---------|-------------|
| `opcode_bit` | Bit de instrucción que codifica operación |
| `register_index_bit` | Bit que selecciona un registro |
| `immediate_bit` | Bit de un valor inmediato |
| `program_counter_bit` | Bit del PC |
| `instruction_bus` | (kind de grupo) Bus que transporta una instrucción |
| `memory_word_bit` | Bit de una palabra de memoria |
| `pc_increment_signal` | Señal de control de avance del PC |

Estas etiquetas no se usan en niveles 0–13 del catálogo inicial salvo en pruebas exploratorias del nivel 13.

---

## Reglas de uso

### U.1 — Etiqueta o vacío

Un puerto o señal puede tener:

- Una **etiqueta única** del catálogo, o
- Etiqueta **vacía** (`""`) si su rol es genérico y no requiere semántica especializada.

No se permiten etiquetas múltiples directamente sobre una sola señal. Si una señal participa en varios roles, se documenta a través de `group` (puede pertenecer a más de un grupo simultáneamente).

### U.2 — Coherencia familia → etiqueta

Cada **familia de circuitos** (ver [22 — Catálogo de casos límite](22-edge-case-catalog.md)) tiene convenciones esperadas:

| Familia | Etiquetas esperadas en salidas |
|---------|-------------------------------|
| Half adder / full adder / sumador N-bit | `sum_bit` (en cada bit de suma), `carry` (cout) |
| Restador | `difference_bit`, `borrow` |
| Comparador | `comparison` o desglose `equal_flag`, `greater_flag`, `less_flag` |
| Multiplexor | `data_bit` (salida y); inputs con `data_bit` y `select` (o `select_bus`) |
| Demultiplexor | `data_bit` en cada salida; input con `data_bit` y `select` |
| Decoder | `select_output` en cada salida; inputs con `select` (o `select_bus`) |
| Encoder | dual del decoder |
| ALU | salidas con `data_bit` + grupo `flags` |
| Registro | salidas con `data_bit`; inputs con `data_bit`, `clock`, `enable` (o `write_enable`) |
| Memoria | inputs: `address_bus`, `data_bus` (in), `clock`, `write_enable`, `read_enable`; outputs: `data_bus` (out) |

Coherencia incompleta no es error; el verificador la marca como **warning auditable**.

### U.3 — Conservación bajo optimización

Las transformaciones del optimizador deben **conservar** las etiquetas semánticas de los puertos del circuito y de cualquier señal interna marcada con etiqueta explícita. La transformación H.2 (`signal_renaming_normalization`) puede renombrar pero **preserva la etiqueta semántica** asignada.

Si una transformación elimina una señal con etiqueta, esa eliminación es legítima solo si la señal era muerta (A.1) o redundante con otra señal que ya lleva una etiqueta equivalente.

### U.4 — Estabilidad del hash canónico

El hash canónico **incluye** las etiquetas semánticas como parte de la topología abstracta. Cambiar la etiqueta de una señal cambia el hash. Esto es deseable: dos circuitos estructuralmente equivalentes pero con semántica diferente son **versiones distintas**.

### U.5 — Etiquetas en el `.aoncir`

En el formato físico (ver [21 — Sintaxis física `.aoncir`](21-aoncir-syntax.md)) las etiquetas aparecen en:

- `[[ports.inputs]]` / `[[ports.outputs]]` (campo `semantic_tag`). Arrays de tablas: el orden de aparición es contrato formal del `InputVector` y `OutputVector`.
- `[[signals]]` (campo `semantic_tag`).
- `[[semantic_groups]]` (campo `kind`).

El parser estricto rechaza:

- Etiquetas no listadas en este catálogo (modo estricto).
- `kind` de grupo no listado.
- Conflictos según las reglas siguientes.

### U.6 — `bit_position` y orden dentro de un bus

Cuando varios puertos pertenecen al mismo `group` cuya `kind` admite agrupación por bits (`bus`, `address_bus`, `data_bus`, `select_bus`, `operand`, `flags` con orden definido), cada puerto declara un campo entero opcional `bit_position` (ver [21 — Sintaxis física `.aoncir` §P.4](21-aoncir-syntax.md)):

- `bit_position` ≥ 0, único dentro del grupo, contiguo desde 0 hasta `width - 1`.
- Para puertos sin `group` o con `kind = "flags"` cuyo orden no está fijado, `bit_position` se omite o se ignora.

`bit_position` es una etiqueta de **orden posicional dentro de un bus**, no un identificador alternativo. No se usa para nombrar señales ni para acceder a puertos por índice fuera del contrato definido por el orden de aparición de `[[ports.inputs]]` / `[[ports.outputs]]`.

### U.7 — Endianness canónico de AONIX (regla normativa)

> **`bit_position = 0` es siempre el LSB (least significant bit).**

Esta es **la convención fija de AONIX**, sin posibilidad de configuración global por instalación.

Reglas operativas que se derivan:

- Dentro de cualquier `[[semantic_groups]]` con `kind` agrupable por bits (`bus`, `address_bus`, `data_bus`, `select_bus`, `operand`), `bit_position = 0` es el **bit menos significativo**.
- A medida que `bit_position` crece, los bits son **más significativos**.
- `bit_position = width - 1` es el **MSB (most significant bit)**.
- El orden debe ser **contiguo desde 0 hasta `width - 1`**, sin huecos ni duplicados.

Razones de fijar esta convención sin parámetro:

1. **Determinismo total.** Un mismo `.aoncir` interpretado en distintas instalaciones produce los mismos resultados aritméticos. No hay flag global de instalación que altere la interpretación.
2. **Coherencia con la aritmética software.** El modelo de referencia software (sumadores, comparadores) usa LSB-first como convención implícita; AONIX lo hace explícito.
3. **Auditoría.** Un revisor humano no necesita conocer la configuración de la instalación para entender el orden de bits de un bus.
4. **Aprendizaje.** Una IA aprende **una sola** convención de endianness; no tiene que adaptarse a distintas.

Consecuencia para el verificador y el simulador:

- Cuando el verificador compara la salida del simulador con un valor aritmético de referencia, **el bit en `bit_position = i` corresponde a `2^i`** del valor aritmético.
- La verificación por señal semántica de buses utiliza esta convención por defecto.

Consecuencia para el parser estricto:

- Un `.aoncir` que declare explícitamente otra convención (por ejemplo, un campo `endianness = "big"` dentro de un `[[semantic_groups]]`) se **rechaza** en modo estricto. AONIX no admite endianness por grupo en la versión 1.0.0 del catálogo.

Esta regla es **normativa para Fase 1** y forma parte del contrato físico del formato. Si en el futuro se necesita soportar otras convenciones (p.ej. por interoperabilidad con sistemas externos), se introducirá vía `format_version` superior con auditoría humana (ver [25 — Política de auditoría humana](25-human-audit-policy.md)), nunca como flag silencioso.

---

## Reglas de conflicto entre etiquetas

### C.1 — Conflictos directos

Ninguna señal puede portar simultáneamente dos etiquetas mutuamente excluyentes. Tabla de conflictos directos:

| Conflicto | Etiquetas |
|----------|-----------|
| Datos vs control | `data_bit` ⇄ `control_signal`, `data_bit` ⇄ `clock`, `data_bit` ⇄ `reset` |
| Acarreo vs préstamo | `carry` ⇄ `borrow` |
| Flags entre sí | dos flags distintas (`zero_flag`, `carry_flag`, etc.) en la misma señal |
| Selección vs dato | `select` ⇄ `data_bit` en la misma señal |
| Temporal vs combinacional puro | `clock` ⇄ cualquier dato dentro de un circuito **no temporal** (rechazado para nivel < 11) |

El sistema no fuerza una sola etiqueta por señal; fuerza que cuando hay etiqueta, sea **una sola** y compatible con su rol declarado.

### C.2 — Conflictos contextuales

Algunas combinaciones son legales en niveles altos y rechazadas en niveles bajos:

| Combinación | Niveles donde es legal | Niveles donde es ilegal |
|------------|------------------------|-------------------------|
| `clock` en un input | ≥ 11 | 0–10 (la tarea no es temporal) |
| `reset` en un input | ≥ 11 | 0–10 |
| `enable`, `write_enable`, `read_enable` | ≥ 11 | 0–10 |
| `address_bus` (como kind de grupo) | ≥ 12 | 0–11 (no hay memorias direccionables) |
| `instruction_bus` y `opcode_bit` | ≥ 13 | 0–12 |

El cargador de tareas y el parser estricto verifican esto.

### C.3 — Conflictos en grupos

Un `semantic_group` declarado con `kind = "flags"` no puede contener miembros con etiquetas no-flag. Análogamente, un grupo `kind = "bus"` no debe mezclar etiquetas que pertenezcan a roles incompatibles.

Tabla de compatibilidad miembro → kind del grupo:

| `kind` del grupo | Etiquetas válidas en miembros |
|------------------|-------------------------------|
| `bus` | `data_bit`, `operand_bit`, `sum_bit`, `difference_bit` (homogéneo dentro del grupo) |
| `address_bus` | `data_bit` (en su rol de bit de dirección, sin etiqueta de operando aritmético) |
| `data_bus` | `data_bit` |
| `select_bus` | `select` |
| `control_bus` | `control_signal`, `clock`, `reset`, `enable`, `write_enable`, `read_enable` (combinaciones permitidas según diseño) |
| `operand` | `operand_bit` |
| `flags` | etiquetas de Categoría 3 únicamente |

Heterogeneidad fuera de la tabla anterior genera **warning auditable** en niveles 0–10 y **error** en niveles ≥ 11 (donde la coherencia del bus afecta la simulación temporal).

---

## Reglas de agrupación: buses, flags y control

### G.1 — Buses

Un **bus** es un grupo lógico de señales que se tratan como conjunto. AONIX modela cada bit del bus como una señal individual; el grupo declara la pertenencia. Reglas:

1. Un bus declara `id`, `kind`, `members`, `width`.
2. `width` = `len(members)`. Si difieren ⇒ error del parser.
3. Los miembros se enumeran en orden **LSB-first canónico**: `bit_position = 0` = LSB, `bit_position = width - 1` = MSB (ver §U.7). Esta convención es fija; no admite declaración alternativa por tarea ni por instalación.
4. Una señal puede pertenecer a **un solo** bus de tipo `bus`/`address_bus`/`data_bus`/`select_bus`/`operand` simultáneamente. Puede pertenecer adicionalmente a un grupo de `kind = "control_bus"` solo si es señal de control compatible.
5. Los buses **no son nodos del grafo lógico**. No tienen "entrada" ni "salida" como compuertas; son agrupaciones declarativas.

### G.2 — Flags

Una flag-set es un grupo `kind = "flags"`. Reglas:

1. Cada miembro lleva una etiqueta única de Categoría 3.
2. La verificación por señal semántica se aplica a cada flag por separado.
3. Una tarea puede declarar la flag-set como obligatoria para la promoción (el oficial activo de una ALU debe declarar su flag-set).
4. El visualizador asigna colores reservados por tipo de flag (ver [09 §Estilo y semántica visual normativos](09-visualization-vulkan.md)).

### G.3 — Control

Las señales de control (`clock`, `reset`, `enable`, `write_enable`, `read_enable`, `control_signal`) pueden:

- Aparecer individualmente como puertos de input del circuito.
- Agruparse en un `control_bus` si la tarea lo justifica.
- No mezclarse con buses de datos: un `data_bus` no contiene un `clock`.

`clock` tiene reglas adicionales:

- En un mismo circuito puede haber **a lo sumo un `clock`** por dominio de reloj. Multi-clock se permitirá en niveles arquitectónicos avanzados con declaración explícita.
- Cualquier señal etiquetada `clock` activa el modo temporal del simulador; el cargador de tareas verifica que la tarea sea de nivel ≥ 11.

---

## Política de extensión

Esta versión 1.0.0 del catálogo es **cerrada** salvo extensión vía namespace explícito declarado en la configuración de instalación. Una etiqueta custom toma la forma `ns:nombre` donde `ns` es un namespace registrado.

Ejemplo (hipotético):

```toml
[[ports.outputs]]
name         = "custom_output"
semantic_tag = "lab1:special_marker"
group        = ""
```

Sin namespace registrado, el parser rechaza etiquetas que no estén en este documento. La extensión por namespace **no** sirve para reintroducir compuertas: una etiqueta es semántica, nunca operacional, y nunca conecta con un nodo lógico de tipo distinto a AND/OR/NOT.

---

## Validación del parser estricto (referencia)

```
SEMANTIC TAG VALIDATIONS

1.  Toda etiqueta en [ports.*], [[signals]] y [[semantic_groups]] pertenece
    al catálogo de este documento o a un namespace registrado.
2.  No hay etiquetas múltiples sobre una sola señal.
3.  Etiquetas que requieren modo temporal solo aparecen en tareas de nivel >= 11.
4.  Grupos respetan la compatibilidad miembro → kind de la tabla C.3.
5.  Buses tienen width = len(members).
6.  A lo sumo un clock por circuito (sin extensión multi-clock declarada).
7.  No hay conflictos directos de C.1.
8.  No hay conflictos contextuales de C.2.
9.  Endianness, cuando aplica, está declarada y es consistente entre
    grupos relacionados (operand_a y operand_b deben coincidir).
```

---

## Garantías del catálogo de etiquetas

- **No introduce primitivas.** Las etiquetas son anotaciones; no son nodos del grafo.
- **Estable** entre versiones del catálogo: las etiquetas existentes no se eliminan ni renombran sin auditoría humana ([25](25-human-audit-policy.md)).
- **Coherencia transversal.** La misma etiqueta en distintos circuitos significa lo mismo.
- **Inspeccionable.** Las etiquetas aparecen explícitas en el `.aoncir`; el revisor humano puede auditarlas.
- **Usable por todos los módulos.** Simulador (verificación semántica), verificador (suites específicas), evaluador (métricas por categoría), visualizador (colores), traductores (explicaciones).

## Lo que las etiquetas **no** hacen

- **No transforman primitivas.** Una señal con etiqueta `carry` sigue siendo un bit producido por compuertas AND/OR/NOT.
- **No introducen comportamiento.** El comportamiento lo dicta la topología del grafo, no la etiqueta.
- **No son fuente de verdad.** El verificador valida que la señal con etiqueta `carry` se comporta como acarreo; si no lo hace, la etiqueta **no salva** al circuito de un FAIL.
- **No son obligatorias** salvo cuando la tarea las exige. Una señal sin etiqueta es legal; simplemente no obtiene verificación semántica.

## Decisiones cerradas en este documento

- Catálogo inicial 1.0.0 con 8 categorías y ~40 etiquetas individuales.
- Reglas de conflicto directo y contextual.
- Reglas de agrupación para buses, flags y control.
- Política de extensión vía namespace.
- **Endianness canónico:** `bit_position = 0` es LSB, `bit_position = width - 1` es MSB. Fijo, sin parámetro (§U.7).

## Decisiones que siguen abiertas

- Lista exacta de namespaces "estándar" reservados (`lab1:`, `cpu_min:`, `mem:`).
- Mecanismo formal de registro de namespaces.
- Política de migración entre versiones del catálogo (por ej. si en versión 2.0.0 se renombra `equal_flag` → `eq_flag`; ¿qué ocurre con los `.aoncir` viejos?).
