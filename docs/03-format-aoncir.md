# 03 — Formato `.aoncir`

## Identidad del formato

**`.aoncir`** — **AON C**ircuit **I**ntermediate **R**epresentation.

AON = AND, OR, NOT.

Es el **documento técnico canónico** del circuito. Representa la verdad estructural oficial.

## Propósito

El `.aoncir` debe ser:

- **Único** como versión oficial activa por circuito y tamaño.
- **Canónico** — para un circuito lógicamente equivalente y con la misma estructura, la serialización produce el mismo hash.
- **Verificable** — el parser comprueba estructura, reglas absolutas y consistencia antes de aceptarlo.
- **Simulable** — directo: el simulador opera sobre el grafo parseado sin transformaciones adicionales.
- **Optimizable** — el evaluador puede leerlo, computar métricas y guiar optimización.
- **Visualizable** — Vulkan renderiza directamente desde la representación parseada.
- **Traducible** — humano e IA.
- **Comparable** — dos `.aoncir` se comparan por estructura, métricas y comportamiento.
- **Auditable** — cada campo es legible y trazable.
- **Expandido completamente a AND/OR/NOT** — el grafo no contiene jerarquía resuelta a primitivas distintas.

## Decisión pendiente: representación física

AONIX necesita una representación física concreta. Hay tres alternativas razonables; la decisión final corresponde al usuario.

### Alternativa A — Texto humano-legible (TOML/YAML/JSON, o DSL propio)

**Pros:** legible, diffable en git, fácil de auditar a mano.
**Contras:** más lento de parsear; tamaños grandes se vuelven pesados; necesita disciplina para mantener canonicidad.

### Alternativa B — Binario compacto

**Pros:** rápido, compacto, hash trivialmente reproducible.
**Contras:** opaco sin herramientas; difícil de auditar manualmente.

### Alternativa C — Doble cara: binario canónico + proyección textual generada

**Pros:** lo mejor de ambos mundos — el binario es la fuente de verdad; la proyección textual es para humanos y diff.
**Contras:** dos paths de parser, complejidad.

**Recomendación de diseño:** empezar con **Alternativa A en formato propio basado en TOML** durante fases iniciales (legibilidad y velocidad de iteración), con migración a **Alternativa C** cuando el sistema madure. La decisión final queda abierta para confirmación.

## Estructura lógica (independiente de representación física)

Un `.aoncir` describe:

```
1. Metadatos
   - nombre canónico (ej. four_bit_full_adder)
   - versión semántica
   - tamaño paramétrico (ej. width = 4)
   - autor (humano | agent_id | ai_model_id)
   - fecha de creación
   - hash canónico
   - hash del predecesor (si esta versión reemplaza una anterior)
   - nivel curricular asociado
   - tarea de origen (opcional)

2. Puertos
   - inputs: [ { name, semantic_tag?, group? } ]
   - outputs: [ { name, semantic_tag?, group? } ]

3. Señales internas
   - [ { id, semantic_tag?, group? } ]

4. Compuertas
   - [ { id, kind ∈ {AND, OR, NOT}, inputs: [signal_id], output: signal_id } ]

5. Conexiones (opcional, si no son derivables de inputs/output de gates)
   - [ { from: signal_id, to: signal_id } ]

6. Etiquetas semánticas (opcional)
   - grupos: [ { name, members: [signal_id], tag: bus|address_bus|data_bus|... } ]

7. Información de layout 2D
   - [ { signal_or_gate_id, x, y } ]      # opcional pero recomendado
   - regiones agrupadas: [ { name, bbox } ]

8. Especificación de comportamiento (referencia, opcional)
   - spec_id o spec inline (tabla de verdad, modelo de referencia)

9. Reporte de verificación
   - pruebas superadas: { suite_id, count, seed, timestamp }
   - métricas de evaluador: gate_count, depth, dead_signals, ...

10. Predecesores (cadena histórica)
    - [ hash_canónico_de_versión_anterior, ... ]
```

## Reglas estructurales del grafo

El grafo extraído de un `.aoncir` debe cumplir, sin excepción:

1. Todo nodo es de tipo `AND`, `OR` o `NOT`. **Cualquier otro tipo invalida el archivo.**
2. Todo `NOT` tiene aridad de entrada exactamente 1.
3. Todo `AND` y `OR` tienen aridad de entrada ≥ 2.
4. Toda señal está definida antes de ser usada (no hay referencias colgantes).
5. Toda señal usada como entrada de algún nodo proviene de:
   - un puerto de entrada del circuito, **o**
   - la salida de otro nodo, **o**
   - una constante explícita (`0` o `1`).
6. Toda salida del circuito se asigna a alguna señal existente.
7. El grafo es un DAG. No hay ciclos salvo cuando el nivel lo permite explícitamente (estructuras de memoria con feedback gobernado por `clock`), y en ese caso la "rotura" del ciclo está documentada en el archivo.
8. No hay señales muertas en la versión oficial activa (las muertas pueden existir en versiones experimentales, pero el verificador rechaza promoverlas a oficiales).
9. Los nombres de señal son únicos dentro del archivo.
10. Los IDs internos son estables y deterministas (forman parte del hash canónico).

## Hash canónico

El hash canónico de un `.aoncir` se calcula sobre:

1. La topología del grafo (nodos + aristas) en orden canónico (ordenamiento topológico determinista con desempate por nombre).
2. El mapeo de puertos.
3. El conjunto de etiquetas semánticas (orden lexicográfico).
4. **No incluye:** layout 2D, metadatos de autor/fecha, ni reporte de verificación.

Dos `.aoncir` con el mismo hash canónico son **estructuralmente equivalentes**. La equivalencia de comportamiento se decide independientemente por el verificador.

## Esqueleto ilustrativo (no normativo)

Solo a fines de comunicar la forma esperada; la sintaxis final se fijará cuando el usuario apruebe la alternativa de representación.

```toml
[meta]
name = "one_bit_full_adder"
version = "1.0.0"
width = 1
level = 7
hash_canonical = "..."
predecessor = []
created_at = "2026-05-11T19:20:00Z"

[ports.inputs]
a       = { semantic_tag = "operand_bit" }
b       = { semantic_tag = "operand_bit" }
cin     = { semantic_tag = "carry" }

[ports.outputs]
sum     = { semantic_tag = "sum_bit" }
cout    = { semantic_tag = "carry" }

[[signals]]
id = "s1"
[[signals]]
id = "s2"
# ... más señales internas

[[gates]]
id = "g1"
kind = "NOT"
inputs = ["b"]
output = "nb"

[[gates]]
id = "g2"
kind = "AND"
inputs = ["a", "nb"]
output = "s1"

# ... el grafo completo expandido a AND/OR/NOT, sin XOR

[verification]
suite = "exhaustive_3_inputs"
passed = 8
total = 8
seed = null

[metrics]
gate_count = { AND = 5, OR = 2, NOT = 2 }
depth = 4
dead_signals = 0
```

## Reglas operativas

1. **Un `.aoncir` oficial activo por circuito y tamaño.** Si existe `four_bit_full_adder.aoncir` oficial, una propuesta nueva solo puede reemplazarlo si pasa el verificador en el nivel correspondiente y mejora bajo los criterios del evaluador.
2. **Reemplazo trazable.** El campo `predecessor` registra el hash canónico de la versión anterior. La versión reemplazada se mueve a memoria histórica (ver [05 — Memorias](05-memory-system.md)).
3. **Nunca se borra historia.** La versión histórica permanece disponible para auditoría, comparación y entrenamiento.
4. **Inmutabilidad del archivo activo.** Un `.aoncir` oficial activo no se edita en sitio. Se crea uno nuevo y se reemplaza atómicamente.
5. **Parser estricto.** Cualquier desviación de las reglas estructurales hace fallar el parser. No hay "modo permisivo".

## Lo que un `.aoncir` **no** contiene

- Información de aprendizaje (eso va en `.aonclg`).
- Trayectoria del agente que lo construyó.
- Recompensas, errores comunes, casos fallidos.
- Comentarios libres sin estructura semántica.
- Compuertas distintas de AND, OR, NOT, en ningún nivel del archivo.

## Validación del parser

Pseudocódigo del flujo de carga:

```
parse(bytes) {
    structure   = deserialize(bytes)             // sintaxis válida del formato
    check_meta(structure.meta)                   // campos obligatorios
    graph       = build_graph(structure)         // nodos y aristas
    enforce_R2(graph)                            // todo nodo ∈ {AND, OR, NOT}
    enforce_arity(graph)                         // aridades correctas por tipo
    enforce_dag(graph)                           // sin ciclos (o ciclos permitidos)
    enforce_no_dangling(graph)                   // sin referencias colgantes
    enforce_named_ports(structure.ports)         // puertos correctamente declarados
    enforce_unique_signal_names(structure)       // nombres únicos
    h           = canonical_hash(graph)          // hash canónico
    return Circuit { meta, graph, hash: h }
}
```

Cualquier fallo aborta la carga y reporta el error con localización exacta.

## Versiones del formato

El formato `.aoncir` lleva un campo `format_version` propio (separado de la versión del circuito). Cambios incompatibles incrementan major. Los parsers retrocompatibles deben soportar versiones anteriores hasta donde sea razonable.
