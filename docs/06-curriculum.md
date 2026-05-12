# 06 — Sistema curricular

## Filosofía

AONIX es un mundo formal **organizado como videojuego**. El aprendizaje progresa por niveles, cada uno con metas claras, pruebas proporcionales y criterios de avance objetivos.

**Regla central:** el avance se gana por **demostración de dominio**, nunca por tiempo, nunca por intervención manual no auditable.

## Anatomía de un nivel

Cada nivel define:

- **Tipos de entradas** permitidas.
- **Tipos de salidas** permitidas.
- **Metas** (clases de tarea que se proponen).
- **Dificultad** (composición de N tareas con N₁ exhaustivas, N₂ aleatorias, etc.).
- **Pruebas requeridas** (qué suites debe superar un circuito para ser aceptado).
- **Criterios de avance** (qué tasa de éxito, qué estabilidad).
- **Métricas mínimas** (umbrales de evaluador: máximo de compuertas tolerable, profundidad razonable).
- **Tareas desbloqueables** al superar el nivel.
- **Condiciones de éxito** del nivel completo.
- **Condiciones de fallo** del nivel (cuándo el agente debe quedarse, no avanzar).

## Anatomía de una tarea

Una tarea es la unidad operativa. Toda tarea tiene:

- `name` — identificador legible.
- `level` — nivel curricular al que pertenece.
- `inputs` — número y etiquetas.
- `outputs` — número y etiquetas.
- `semantic_tags` — opcionales.
- `expected_behavior` — spec formal (tabla de verdad, propiedad, modelo).
- `required_tests` — referencias a suites en memoria de pruebas.
- `success_criteria` — qué exige superar.
- `evaluated_metrics` — qué mide el evaluador (con pesos).
- `initial_state` — circuito parcial inicial (usualmente vacío).
- `allowed_actions` — subconjunto de acciones permitidas en este nivel.

**Restricción absoluta:** una tarea **nunca le dice al sistema qué compuerta derivada usar**. La tarea declara la meta; descubrir la estructura es trabajo del agente.

### Ejemplo correcto

```
Tarea: comportamiento de acarreo de un bit

Entradas:
  first_input_bit
  second_input_bit
Salida:
  carry_output   (semantic_tag: carry)
Meta:
  La salida debe activarse solo cuando ambas entradas estén activas.
Primitivas disponibles:
  AND, OR, NOT
```

La tarea **no dice** "usa AND". Solo define la meta. El agente descubre que `carry_output = first_input_bit AND second_input_bit`.

## Progresión de niveles

### Nivel 0 — Mundo formal

**Propósito:** representar el mundo. No se construyen circuitos completos todavía; se aprende el vocabulario.

**Conceptos introducidos:**
- Señales (entradas, salidas, intermedias).
- Compuertas primitivas AND, OR, NOT.
- Concepto de circuito como grafo 2D.
- Concepto de tarea, estado, prueba.

**Tareas tipo:** identificar componentes, leer un `.aoncir` mínimo, simular una entrada, leer una tabla de verdad pequeña.

**Pruebas:** comprensión correcta del modelo.

### Nivel 1 — Lógica de una entrada

**Propósito:** construir comportamientos triviales con una entrada.

**Tareas tipo:**
- `Y = A` (puente trivial — no requiere compuerta).
- `Y = NOT A`.
- `Y = constante 0`.
- `Y = constante 1`.

**Pruebas:** exhaustiva (2 combinaciones).

### Nivel 2 — Lógica de dos entradas

**Propósito:** resolver las funciones booleanas de dos entradas como tareas. **Sin convertir derivadas en primitivas.**

**Tareas tipo:**
- `Y = A AND B`.
- `Y = A OR B`.
- `Y = NOT (A AND B)` (NAND como comportamiento).
- `Y = NOT (A OR B)` (NOR como comportamiento).
- `Y = (A AND NOT B) OR (NOT A AND B)` (XOR como comportamiento).
- `Y = NOT ((A AND NOT B) OR (NOT A AND B))` (XNOR como comportamiento).

Las 16 funciones booleanas de dos entradas son tarea. Las "compuertas derivadas" aparecen como **comportamiento descubierto**, nunca como primitivas. Esta distinción es la lección central del nivel 2.

**Pruebas:** exhaustiva (4 combinaciones) + casos límite (todo-cero, todo-uno).

### Nivel 3 — Lógica de tres o más entradas

**Propósito:** combinatoria amplia y comienzo de pruebas más serias.

**Tareas tipo:**
- Mayoría de tres (`Y = (A AND B) OR (A AND C) OR (B AND C)`).
- Paridad de tres (XOR de tres como comportamiento).
- Detección de patrón específico en 3-4 bits.

**Pruebas:** exhaustiva mientras sea viable (8 a 16 combinaciones); empieza a aparecer reutilización de subexpresiones.

### Nivel 4 — Multi-salida

**Propósito:** circuitos con varias salidas que pueden compartir señales internas. Aparece reutilización deliberada.

**Tareas tipo:**
- Half adder (`sum`, `carry`) — ambas salidas comparten subexpresiones.
- Comparador de 1 bit (`A=B`, `A>B`, `A<B`).
- Decodificador 2-a-4.

**Pruebas:** exhaustiva + verificación por señal semántica (`sum` se comprueba contra spec de suma; `carry` contra spec de acarreo).

### Nivel 5 — Circuitos nombrados simples

**Propósito:** construir entidades con identidad. El circuito gana nombre canónico y se vuelve guardable como `.aoncir`.

**Tareas tipo:**
- Multiplexor 2-a-1.
- Multiplexor 4-a-1.
- Demultiplexor 1-a-2.
- Full adder de 1 bit.

**Pruebas:** exhaustiva + casos límite + suite de regresión inicial. Primera vez que aparece **memoria canónica** para los resultados.

### Nivel 6 — Buses

**Propósito:** agrupar señales semánticamente. Las etiquetas `bus`, `main_bus`, `address_bus`, `data_bus` informan al simulador, visualizador y verificador.

**Tareas tipo:**
- Multiplexor de buses de 4 bits.
- Selector de bus.
- Demultiplexor de buses.

**Restricción:** los buses **no son una operación lógica nueva**. Son agrupación visual y semántica sobre señales que siguen siendo bits AND/OR/NOT-eables.

**Pruebas:** exhaustivas mientras sea viable + casos límite por bus (bus todo-cero, todo-uno, alternado).

### Nivel 7 — Aritmética pequeña

**Propósito:** sumadores, restadores, propagación de acarreo.

**Tareas tipo:**
- Full adder de 1 bit (refinamiento del nivel 5 con métricas más estrictas).
- Sumador de 2 bits con ripple carry.
- Restador de 2 bits.
- Comparador de 2 bits.

**Pruebas:** exhaustiva (16 a 256 combinaciones) + casos límite + comparación contra modelo aritmético de referencia.

### Nivel 8 — Aritmética ampliada

**Propósito:** versiones multi-bit. La parametrización aparece de verdad: `width = 4, 8, 16, 32`.

**Tareas tipo:**
- `four_bit_full_adder`.
- `eight_bit_full_adder`.
- `sixteen_bit_full_adder`.
- `thirty_two_bit_full_adder`.
- Restadores correspondientes.
- Comparadores N-bit.

**Pruebas:**

- N pequeño (4, 8 bits): exhaustiva si viable.
- N mayor (16, 32 bits): aleatoria con semilla + casos límite + comparación con modelo + regresión.

### Nivel 9 — Flags

**Propósito:** señales semánticas de estado aritmético.

**Tareas tipo:**
- Generar `zero_flag`, `carry_flag`, `overflow_flag`, `negative_flag` a partir de un sumador/restador.
- Verificar que las flags se activan exactamente cuando deben.

**Pruebas:** verificación por señal semántica + casos límite específicos (overflow positivo y negativo, cero exacto, etc.).

### Nivel 10 — ALU mínima

**Propósito:** combinar varias operaciones en una unidad seleccionable por opcode.

**Tareas tipo:**
- ALU 4-bit que ofrece: AND, OR, NOT, suma, resta (la unidad las ofrece **como operaciones implementadas con AND/OR/NOT**, no como primitivas nuevas).
- Selector de operación con flags asociadas.

**Pruebas:** exhaustiva por (opcode, operandos) si viable + aleatoria masiva + regresión + verificación por flag.

### Nivel 11 — Estado y reloj

**Propósito:** introducir el tiempo discreto. Las señales etiquetadas `clock`, `reset`, `enable` adquieren semántica temporal.

**Cuidado conceptual:** `clock` no es una primitiva. Es una señal cuyo **uso** activa el modo temporal del simulador.

**Tareas tipo:**
- Latch SR construido con NOR-equivalente expandido a AND/OR/NOT.
- D-latch.
- D-flip-flop (con `clock`).

**Pruebas:** secuencias temporales reproducibles + verificación de propiedades temporales (estabilidad, glitch-freedom dentro del modelo).

### Nivel 12 — Memoria lógica

**Propósito:** estructuras de almacenamiento y lectura.

**Tareas tipo:**
- Registro de N bits.
- Banco de registros pequeño.
- Memoria direccionable mínima.

**Pruebas:** secuencias de escritura/lectura + casos límite (dirección 0, dirección máxima, escritura repetida, lectura sin escritura previa) + regresión.

### Nivel 13 — Arquitectura mínima

**Propósito:** CPU mínima simulable por partes.

**Tareas tipo:**
- Datapath simple (PC, registro, ALU, memoria).
- Decodificador de instrucciones mínimo.
- Ejecución de programa de 3-5 instrucciones.

**Pruebas:** programas pequeños + verificación post-ejecución del estado de registros y memoria.

## Niveles posteriores (visión futura, no obligatorios fase 1)

- Nivel 14 — Pipeline (segmentación, hazards de datos).
- Nivel 15 — Forwarding y stalls.
- Nivel 16 — Cachés.
- Nivel 17 — Predicción de branches.
- Nivel 18 — Arquitecturas multinúcleo.

Estos niveles existen como visión a largo plazo. No son alcance de la fase inicial.

## Criterios de avance curricular

El sistema solo avanza de nivel cuando cumple condiciones medibles:

1. **Tasa de éxito suficiente** sobre tareas del nivel (umbral configurable, p.ej. ≥ 95% de tareas superadas).
2. **Estabilidad en pruebas aleatorias** (mismo agente, mismas tareas, distintas semillas, mismo nivel de éxito).
3. **Ausencia de errores sistemáticos** (no fallar siempre el mismo tipo de caso límite).
4. **Reducción de acciones inválidas** (el validador rechaza menos del X% de acciones propuestas).
5. **Optimización estructural aceptable** (las soluciones no son grotescamente subóptimas; métricas del evaluador dentro de umbrales).
6. **Capacidad de explicar resultados** (el traductor humano puede generar explicaciones; el agente puede responder preguntas estructurales sobre sus soluciones).
7. **Desempeño consistente en regresiones** (no romper lo que ya funcionaba).
8. **Dominio de casos límite** específicos del nivel.
9. **Validación de señales semánticas** cuando aplique (los `carry`, `zero`, `overflow` se comportan como deben).
10. **Mejora frente a versiones históricas** cuando una tarea ya tiene oficial activo.
11. **Capacidad de superar pruebas proporcionales al nivel** completas.

Ninguno de estos criterios es individualmente suficiente. El conjunto es el filtro.

## Quién puede avanzar

- **Humanos** — el avance es informativo (qué desbloquea).
- **Búsqueda exhaustiva / SAT / heurísticas** — útil como baseline.
- **IA** — el caso central. Cada agente tiene su propio progreso curricular.

Los progresos no se mezclan automáticamente entre agentes. Cada uno demuestra dominio por su cuenta. (Se pueden compartir `.aoncir` y `.aonclg`, pero **el aprendizaje** es individual; ver el principio rector.)

## Política de retroceso

Si un agente, tras avanzar, empieza a fallar tareas que dominaba, el sistema **no le quita el avance**, pero registra la regresión y puede:

- Reabrir niveles previos como tareas de refresco.
- Marcar al agente como inestable.
- Pedir auditoría del modelo.

El avance no se anula porque el avance es un hecho histórico (se ganó en su momento); pero la habilidad puede degradarse y el sistema lo detecta.
