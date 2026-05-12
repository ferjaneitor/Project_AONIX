# 13 — Reglas de aceptación de un circuito

> **Documento normativo.** Lista cerrada de condiciones que un circuito propuesto debe satisfacer para ser **aceptado** por AONIX, ya sea como solución correcta de una tarea o como candidato a versión oficial activa. Sin todas estas condiciones, no hay aceptación. No hay excepciones por contexto, autor, recompensa, ni narrativa.

## Resumen

Un circuito propuesto por un agente pasa por dos puertas de aceptación, en orden:

1. **Puerta de validez estructural** (gestionada por validador y parser).
2. **Puerta de corrección funcional y calidad** (gestionada por verificador y evaluador).

Solo si pasa **las dos** puede ser considerado solución aceptada. Solo si pasa **las dos y mejora estrictamente** al oficial activo puede sustituirlo en memoria canónica.

## Puerta 1 — Validez estructural

Un circuito es **estructuralmente válido** si cumple **todas** las siguientes condiciones:

### A. Conformidad con las reglas absolutas

1. Todos los nodos del grafo son de tipo `AND`, `OR` o `NOT`. **No hay excepciones.**
2. Las aridades son correctas:
   - `NOT` con exactamente 1 entrada.
   - `AND` y `OR` con al menos 2 entradas.
3. El circuito está expresado en un entorno 2D (las posiciones de layout, si están presentes, son coordenadas 2D válidas).
4. Si el circuito declara o referencia subcircuitos compuestos, su expansión completa a primitivas no introduce ningún nodo distinto a AND/OR/NOT.

### B. Conformidad de grafo

5. Toda señal usada como entrada de algún nodo está definida (proviene de un puerto de entrada, de la salida de otro nodo, o de una constante explícita `0` o `1`).
6. Toda salida del circuito está conectada a una señal existente.
7. Los identificadores de señal son únicos.
8. El grafo es un **DAG**. Excepción: en circuitos con feedback temporal gobernado por señales etiquetadas `clock`, se permite el ciclo siempre que la rotura del ciclo (gating temporal) esté explícitamente documentada en el `.aoncir` y validada por el modo temporal del simulador.
9. No hay señales colgantes (referencias a IDs inexistentes).

### C. Conformidad del archivo `.aoncir`

10. El archivo se parsea sin errores bajo el parser estricto.
11. El campo `format_version` está presente y es compatible con la versión activa del sistema.
12. Los metadatos obligatorios están completos (`name`, `version`, `width` si aplica, `level`, `created_at`).
13. El hash canónico calculado coincide con el declarado (si lo hay), o se acepta el computado si no estaba declarado.

### D. Conformidad operativa

14. El circuito puede simularse: el simulador ejecuta al menos un vector de entrada y produce un resultado determinista.
15. El circuito puede visualizarse 2D: el visualizador genera al menos un layout válido sin errores.
16. El circuito puede traducirse a estado para IA: el traductor produce el estado conforme al esquema.

Cualquier violación de A–D abortará el procesamiento con error y mensaje específico.

## Puerta 2 — Corrección funcional y calidad

Un circuito estructuralmente válido es **funcionalmente aceptado** como solución de una tarea si cumple **todas** las siguientes condiciones (definidas con detalle en [07 — Pruebas y verificación](07-testing-and-verification.md) y [12 — Especificación formal de tareas](12-task-specification.md)):

### A. Verificación

17. **El verificador entrega `PASA`** sobre todas las suites referenciadas por la tarea (`required_test_suites`).
18. **Cobertura exhaustiva** cuando el espacio de entradas lo permite (decisión por umbral del nivel).
19. **Pruebas aleatorias reproducibles** superadas con la tasa mínima declarada en `success_criteria.minimum_pass_rate_random`.
20. **Casos límite catalogados** todos superados (o todos excepto los marcados como `non_blocking` por la tarea).
21. **Suite de regresión** del circuito (y de su familia) superada.
22. **Validación por señal semántica** (los `carry`, `zero`, `overflow`, `negative`, etc. se comportan según su etiqueta) si la tarea las declara.
23. **Validación temporal** correcta si la tarea es temporal: secuencias de ciclos correctas, reset correcto, enable correcto.
24. **Comparación con modelo de referencia** correcta si la tarea referencia un modelo (`reference_model` o `ReferenceCircuit`).

### B. Calidad estructural mínima

25. **Umbrales del evaluador** cumplidos: las métricas declaradas en `metric_thresholds` están dentro del rango aceptable.
26. **Sin señales muertas** en la versión candidata a oficial activa (las experimentales pueden tener señales muertas; las oficiales activas, no).

### C. Sin regresión contra oficial activo

27. Si existe un oficial activo, el candidato **no debe fallar ninguna prueba que el oficial supera**. Toda regresión es causal de rechazo automático, incluso si el candidato es estructuralmente mejor.

## Puerta de promoción a oficial activo

Pasar las puertas 1 y 2 hace al circuito **aceptado como solución**. Para convertirse en **oficial activo** debe cumplir además:

### Reemplazo de un oficial activo existente

28. **Mejora estricta** según el `ranking_order` del evaluador declarado por la tarea (orden lexicográfico configurable, por defecto: `gate_count, depth, dead_signals, fan_out_max, ...`). Empate ⇒ no se reemplaza; gana el incumbente por estabilidad.
29. **Verificación de la versión optimizada** posterior a la optimización estructural automática (la pipeline aplica el optimizador y re-verifica antes de proponer reemplazo).
30. **Reemplazo atómico transaccional** completado sin error en memoria canónica.

### Primera versión (no hay oficial activo previo)

31. Si no existe oficial activo para esa tarea+tamaño, basta cumplir puertas 1 y 2 para promoción directa.

## Aceptación de circuitos compuestos como entidad canónica

Para que un circuito compuesto (mux, full adder, ALU, registro, etc.) sea aceptado como **entidad canónica guardable** en memoria canónica:

32. Cumple las puertas 1 y 2.
33. Su grafo interno está **completamente expandido a AND/OR/NOT** (no hay nodos opacos de subcircuitos sin expandir en la representación canónica).
34. Posee un **nombre canónico** del catálogo (ej. `four_bit_full_adder`).
35. Posee al menos una `Specification` formal (tabla, propiedades, función de referencia, circuito de referencia, o temporal).
36. Ha pasado la suite de pruebas asociada a su nivel.
37. Tiene `.aoncir` válido con todos los metadatos obligatorios.

Un circuito compuesto **nunca** se acepta como entidad canónica si:

- Aparece como nodo opaco no expandido.
- Su nombre canónico colisiona con primitivas (no puede llamarse `XOR_gate`, `NAND_gate`, etc.).
- Sus puertos no están etiquetados conforme a la convención semántica del catálogo.

## Aceptación de un `.aonclg`

Un `.aonclg` se acepta para registro en memoria de aprendizaje si:

38. Referencia un `.aoncir` válido (que puede ser final, intermedio o histórico) por hash canónico.
39. Cumple el esquema definido en [04 — Formato .aonclg](04-format-aonclg.md).
40. No introduce primitivas ni modifica el grafo del `.aoncir` referenciado.
41. Su trayectoria es consistente con las decisiones del validador (toda acción listada como legal debe tener `validator_feedback: legal`).
42. Está cerrado: el episodio ha terminado con resultado registrado.

## Lista de no negociables

Las siguientes condiciones son **inviolables**. Cualquier intento de aceptar un circuito que las viole es un fallo del sistema, no un caso a discutir:

- **Solo AND/OR/NOT** como tipos de nodo.
- **Una sola versión oficial activa** por circuito+tamaño.
- **El verificador decide correctitud**, ningún otro módulo.
- **El evaluador mide, no decide.**
- **Recompensa alta no compensa fallo del verificador.**
- **Mejora visual o de narrativa no compensa regresión funcional.**
- **Memoria experimental no es atajo** hacia memoria canónica.

## Mecanismos de aplicación

Las reglas de aceptación se aplican en:

1. **Parser de `.aoncir`** (puertas A, C parciales).
2. **Validador de acciones** (puerta A, durante la construcción).
3. **Verificador** (puerta 2 A).
4. **Evaluador** (puerta 2 B).
5. **Coordinador** (orquesta puertas, gestiona promoción atómica).
6. **Memoria canónica** (rechaza escritura que no proviene de promoción del coordinador).
7. **Tests del propio AONIX** (regresiones sobre estas reglas; si alguna falla, el build falla).

## Trazabilidad de la aceptación

Cada aceptación queda registrada con:

- Hash canónico del circuito aceptado.
- Identificadores y versiones de las suites superadas.
- Snapshot de las métricas del evaluador.
- Decisión binaria del verificador.
- Comparación con oficial activo previo, si existió.
- Decisión del coordinador (promoción / no promoción).
- Timestamp y agente.

Sin trazabilidad, no hay aceptación.
