# 01 — Reglas absolutas

## Declaración formal

AONIX tiene **exactamente dos** restricciones absolutas. Son inviolables, no admiten excepciones, no caducan, no pueden modificarse por una IA, por un usuario, por una rama experimental, ni por ningún módulo del sistema.

### R1 — Mundo 2D

El sistema es un entorno lógico y visual **2D**.

Implicaciones:

- Todo circuito se representa como un grafo planar o cuasi-planar embebido en un plano 2D.
- Toda visualización es 2D (no hay 3D, no hay isometría).
- Las coordenadas, posiciones, agrupaciones, buses y rutas se expresan en términos 2D.
- El layout puede ser jerárquico (subgrafos colapsables) pero la proyección final siempre es 2D.

### R2 — Primitivas lógicas únicamente AND, OR, NOT

Las únicas compuertas lógicas **primitivas** son:

- `AND` (aridad estricta: exactamente 2 entradas)
- `OR` (aridad estricta: exactamente 2 entradas)
- `NOT` (aridad estricta: exactamente 1 entrada)

> **Nota normativa sobre aridad (Fase 1).** AONIX usa **aridad estricta binaria** para `AND` y `OR`. Un `AND` de 3 entradas debe construirse como composición de dos `AND` de 2 entradas; lo mismo para `OR`. Esto preserva la honestidad de las métricas de conteo, profundidad y costo: cada `AND` real cuenta como uno; un agente que quiera combinar más entradas paga el precio estructural correspondiente. Una eventual extensión a versiones N-arias en una `format_version` futura sería un cambio auditado del catálogo (severidad S3 en [25 — Política de auditoría humana](25-human-audit-policy.md)), nunca silencioso.

**No existen como primitivas** y **no pueden existir como operaciones invocables**:

- `XOR`
- `XNOR`
- `NAND`
- `NOR`
- Ninguna otra compuerta lógica derivada
- Ninguna operación lógica "atómica" que no sea AND/OR/NOT

## Distinción central (no negociable)

AONIX distingue de forma estricta tres categorías. Confundirlas es violar las reglas absolutas.

### 1. Compuerta primitiva

Únicamente `AND`, `OR`, `NOT`. Es el conjunto cerrado de átomos lógicos. **Cardinalidad: 3.** Inalterable.

### 2. Compuerta lógica derivada

`XOR`, `XNOR`, `NAND`, `NOR`, y cualquier otra función booleana que pueda expresarse a partir de las primitivas.

Las derivadas:

- **Pueden ser descubiertas como comportamiento.** Por ejemplo, un circuito que implementa `Y = (A AND NOT B) OR (NOT A AND B)` exhibe el comportamiento XOR. Esto es legítimo y deseado.
- **Pueden ser metas de tarea.** Una tarea puede pedir al agente construir un circuito cuya tabla de verdad coincida con la del XOR.
- **Pueden ser registros experimentales o ejemplos pedagógicos.**
- **No pueden existir como compuertas disponibles en el inventario de primitivas.**
- **No pueden existir como acciones invocables del agente** (no hay acción `use_xor`).
- **No pueden aparecer como nodos del grafo de un `.aoncir` final.**

### 3. Circuito compuesto guardable

Entidades de mayor nivel construidas combinando primitivas y, opcionalmente, instancias de otros circuitos compuestos previamente verificados. Son la unidad de composición de AONIX.

Pueden guardarse como `.aoncir`:

- Multiplexores y demultiplexores
- Decodificadores y codificadores
- Comparadores
- Half adders y full adders
- Sumadores y restadores de N bits
- Unidades de flags
- ALUs
- Registros
- Bloques de memoria
- Componentes arquitectónicos superiores
- CPUs mínimas
- Bloques arquitectónicos avanzados

**Requisito absoluto:** la red interna del circuito compuesto, cuando se expande, debe estar compuesta exclusivamente por nodos `AND`, `OR` y `NOT`. La jerarquía es una conveniencia de organización y visualización; la verdad estructural es la expansión a primitivas.

## Ejemplos

### Válidos

Archivos:

```
multiplexer_2_to_1.aoncir
one_bit_full_adder.aoncir
two_bit_full_adder.aoncir
four_bit_full_adder.aoncir
eight_bit_full_adder.aoncir
sixteen_bit_full_adder.aoncir
thirty_two_bit_full_adder.aoncir
four_bit_comparator.aoncir
minimal_arithmetic_logic_unit.aoncir
```

Comportamientos descubiertos sin convertirse en primitiva:

```
Tarea: construir comportamiento XOR de dos entradas
Primitivas disponibles: AND, OR, NOT
Solución aceptada: Y = (A AND NOT B) OR (NOT A AND B)
Resultado: tarea superada; XOR no se añade al inventario.
```

### Inválidos

- Guardar `XOR` como compuerta primitiva.
- Permitir una acción `use_xor`.
- Permitir `Y = XOR(A, B)` como operación final en un `.aoncir`.
- Tratar `NAND` como compuerta disponible.
- Tratar `NOR` como compuerta disponible.
- Tratar `XNOR` como compuerta disponible.
- Importar una biblioteca externa que exponga primitivas adicionales.
- Generar un `.aoncir` cuyo grafo contenga un nodo de tipo distinto a AND/OR/NOT.

## Lo que las reglas **no** prohíben

- **No prohíben circuitos complejos.** AONIX está diseñado para escalar hasta CPUs.
- **No prohíben jerarquía.** Los circuitos compuestos son una unidad de composición legítima.
- **No prohíben etiquetas semánticas.** Una salida puede marcarse como `carry`, una entrada como `clock`, un grupo como `bus`. Las etiquetas no son operaciones nuevas.
- **No prohíben temporalidad.** En niveles avanzados, AONIX modela `clock`, `reset` y `enable` como señales etiquetadas; el comportamiento temporal emerge de circuitos lógicos sobre esas señales, no de primitivas nuevas.
- **No prohíben memoria histórica.** Las versiones anteriores de un circuito se conservan para auditoría, comparación y aprendizaje. La memoria histórica no introduce primitivas.
- **No prohíben optimización.** La optimización agresiva es deseada y obligatoria, siempre preservando comportamiento.

## Mecanismos de cumplimiento

Las reglas absolutas se hacen cumplir en múltiples puntos del pipeline:

1. **Validador de acciones.** Rechaza cualquier acción que intente crear un nodo de tipo distinto a AND/OR/NOT, o que invoque una operación derivada.
2. **Parser de `.aoncir`.** Falla al cargar un archivo cuyo grafo contenga nodos prohibidos.
3. **Verificador.** Niega el sello oficial a cualquier circuito que no respete R1 y R2.
4. **Memoria canónica.** Rechaza guardar como oficial activo cualquier circuito que no esté expandido a AND/OR/NOT.
5. **Coordinador central.** Aborta cualquier episodio que intente saltarse el validador.
6. **Traductor para IA.** Nunca expone como acción legal una operación derivada.
7. **Suite de tests del propio AONIX.** Tests de regla absoluta que fallan el build si algún módulo viola R1 o R2.

## Por qué estas reglas existen

El objetivo de AONIX no es facilitar la síntesis de circuitos. El objetivo es **forzar a la IA a aprender lógica booleana profunda desde los átomos**. Permitir compuertas derivadas convertiría AONIX en una biblioteca de atajos donde la IA aprendería a combinar bloques opacos en lugar de descubrir estructura.

La restricción es la pedagogía. La pedagogía es la restricción.

Si AONIX permitiera XOR como primitiva, una IA aprendería a usar XOR. Si AONIX exige construir XOR desde AND/OR/NOT, una IA aprende **qué es** XOR, cómo emerge, dónde se factoriza, cómo se reutiliza y cómo se optimiza. La diferencia entre las dos lecciones es la diferencia entre memorizar y comprender.
