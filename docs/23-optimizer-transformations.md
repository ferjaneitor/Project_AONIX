# 23 — Lista inicial de transformaciones del optimizador

> **Documento normativo.** Catálogo formal de transformaciones que el optimizador estructural puede aplicar. Complementa a [15 — Reglas de optimización](15-optimization-rules.md): allí se definen principios y pipeline; aquí se enumeran transformaciones concretas con su contrato. **No se implementan algoritmos**; se especifican.
>
> **Garantía global.** Toda transformación de este catálogo preserva el comportamiento del circuito sobre el dominio relevante de la tarea. La preservación se asegura por **dos vías obligatorias**: vía algebraica (la transformación corresponde a una ley booleana o equivalencia probada) y vía verificación (tras aplicarla, el verificador re-ejecuta las suites de la tarea). Si la verificación falla, la transformación se descarta y se anota la causa en la memoria de optimización.

## Plantilla por transformación

```
id              identificador único snake_case
name            nombre legible
goal            métrica que típicamente mejora
preconditions   patrón estructural y/o condiciones que deben cumplirse
can_change      qué partes del grafo pueden modificarse
must_not_change qué invariantes están protegidas
required_tests  qué pruebas obligatorias se re-ejecutan post-aplicación
equivalence_risk evaluación del riesgo de romper equivalencia
                 (low / medium / high) con justificación
category        categoría dentro del catálogo
introduced_in   versión del catálogo
notes           observaciones operativas
```

---

# Categoría A — Eliminaciones

Las eliminaciones reducen el tamaño del grafo sin alterar comportamiento. Son las transformaciones con menor riesgo y mayor impacto inmediato.

## A.1 — `dead_signal_elimination`

```
id              dead_signal_elimination
name            Eliminación de señales muertas
goal            ↓ dead_signals, ↓ signals_internal, ↓ gate_count si una
                señal muerta era la única salida de alguna compuerta
preconditions   Existe al menos una señal S tal que no alcanza ninguno
                de los puertos de salida del circuito.
can_change      - Eliminar S del grafo.
                - Eliminar la compuerta que produce S si esa compuerta
                  no alimenta ninguna otra señal viva.
                - Repetir transitivamente.
must_not_change - Conjunto de puertos de entrada y salida.
                - Comportamiento sobre cualquier entrada (las salidas
                  no dependen de S por construcción).
                - Etiquetas semánticas de señales no eliminadas.
                - Hash canónico se recalcula tras la eliminación.
required_tests  Todas las suites de la tarea: exhaustiva (si viable),
                aleatoria con semilla, casos límite blocking,
                regresión, propiedades, semánticas.
                Re-verificación COMPLETA, no parcial.
equivalence_risk low — por definición, S no afecta salidas.
category        A — Eliminaciones
introduced_in   1.0.0
notes           La transformación es transitiva: eliminar S puede
                hacer muertas a señales que solo alimentaban a S.
                Iterar hasta punto fijo.
```

## A.2 — `redundant_gate_elimination`

```
id              redundant_gate_elimination
name            Eliminación de compuertas redundantes
goal            ↓ gate_count
preconditions   Existe una compuerta G cuya salida es:
                  (a) idéntica a una señal ya disponible, o
                  (b) constante por construcción (kind y entradas
                      determinan un valor fijo, ej. AND(x, 0) = 0).
can_change      - Reemplazar todas las referencias a la salida de G
                  por la señal equivalente o la constante.
                - Eliminar G y, transitivamente, las señales muertas
                  resultantes.
must_not_change - Comportamiento sobre cualquier entrada (la equivalencia
                  es algebraica).
                - Puertos del circuito.
                - Etiquetas semánticas de las señales que sobreviven.
required_tests  Suite completa de la tarea + casos límite.
equivalence_risk low — la equivalencia es algebraica directa.
category        A — Eliminaciones
introduced_in   1.0.0
notes           Casos triviales que entran aquí:
                  AND(x, x)  → x
                  OR(x, x)   → x
                  AND(x, 1)  → x
                  OR(x, 0)   → x
                  AND(x, 0)  → 0
                  OR(x, 1)   → 1
                  AND(x, NOT x) → 0
                  OR(x, NOT x)  → 1
```

## A.3 — `double_negation_elimination`

```
id              double_negation_elimination
name            Eliminación de doble negación
goal            ↓ gate_count, ↓ depth
preconditions   Existe una cadena NOT(NOT(x)) — dos compuertas NOT
                consecutivas cuya entrada es x.
can_change      - Reemplazar la salida final por x.
                - Eliminar las dos compuertas NOT.
                - Eliminar la señal intermedia si queda muerta.
must_not_change - Comportamiento sobre cualquier entrada (NOT NOT x = x).
                - Etiquetas semánticas heredadas.
required_tests  Suite completa de la tarea.
equivalence_risk low — ley booleana clásica.
category        A — Eliminaciones
introduced_in   1.0.0
notes           Atención a fan-out: si la salida intermedia NOT(x) es
                consumida por otros nodos además del segundo NOT, no
                puede eliminarse el primer NOT (solo el segundo NOT
                puede sustituirse por x si era el único consumidor).
```

---

# Categoría B — Leyes booleanas básicas

Aplicaciones directas de identidades del álgebra de Boole. Bajo riesgo, alta frecuencia.

## B.1 — `idempotence`

```
id              idempotence
name            Idempotencia
goal            ↓ gate_count
preconditions   Existe AND(x, x) o OR(x, x) (en cualquier forma, incluso
                expandida en árbol).
can_change      - Sustituir AND(x, x) → x.
                - Sustituir OR(x, x) → x.
                - Eliminar compuertas y señales resultantes muertas.
must_not_change - Comportamiento sobre cualquier entrada.
                - Otros consumidores de las señales.
required_tests  Suite completa de la tarea.
equivalence_risk low — identidad clásica.
category        B — Leyes booleanas
introduced_in   1.0.0
notes           Útil cuando la construcción del agente repite una entrada
                accidentalmente.
```

## B.2 — `identity`

```
id              identity
name            Elementos identidad
goal            ↓ gate_count
preconditions   Existe una compuerta cuya OTRA entrada es
                demostrablemente equivalente a 1 (para AND) o a 0
                (para OR) por análisis algebraico (típicamente, una
                rama del propio circuito que cumple x OR NOT x ≡ 1, o
                x AND NOT x ≡ 0).
can_change      Redirigir consumidores hacia la entrada no-identidad x.
                Eliminar la compuerta original si queda sin consumidores.
must_not_change Comportamiento.
required_tests  Suite completa de la tarea.
equivalence_risk low.
category        B — Leyes booleanas
introduced_in   1.0.0
notes           AONIX no admite constantes como SignalReference primitiva
                en Fase 1, por lo que el "1" o "0" de esta regla no es
                un nodo del .aoncir sino una equivalencia detectada por
                el análisis. Combina bien con B.5 (complemento).
```

## B.3 — `annihilation`

```
id              annihilation
name            Elementos aniquiladores
goal            ↓ gate_count y simplificación local
preconditions   Existe una compuerta cuya OTRA entrada es
                demostrablemente equivalente a 0 (para AND, resultado
                trivial 0) o a 1 (para OR, resultado trivial 1).
can_change      Sustituir el subgrafo que produce esa compuerta por
                una rama del circuito que produce el valor aniquilador
                de forma equivalente, eliminando compuertas que dejan
                de ser necesarias.
must_not_change Comportamiento. Otros consumidores de las señales
                originales si los hay.
required_tests  Suite completa.
equivalence_risk low.
category        B — Leyes booleanas
introduced_in   1.0.0
notes           Como en B.2, "0" y "1" son propiedades emergentes del
                análisis, no SignalReferences. La transformación nunca
                produce un nodo constante en el .aoncir resultado.
```

## B.4 — `absorption`

```
id              absorption
name            Absorción
goal            ↓ gate_count, ↓ depth
preconditions   Existe patrón AND(x, OR(x, y)) o OR(x, AND(x, y)).
can_change      Reemplazar el patrón completo por x.
must_not_change Comportamiento.
                Otros consumidores de OR(x, y) o AND(x, y) si los hay.
required_tests  Suite completa.
equivalence_risk low.
category        B — Leyes booleanas
introduced_in   1.0.0
notes           Atención al fan-out: si OR(x, y) tiene otros consumidores
                que se quieren preservar, la transformación solo elimina
                el wrap AND/OR del consumidor concreto, no la señal interna.
```

## B.5 — `complement`

```
id              complement
name            Complemento
goal            ↓ gate_count, simplificación
preconditions   Existe AND(x, NOT x) o OR(x, NOT x) (directamente o tras
                normalización).
can_change      AND(x, NOT x) → 0 ; OR(x, NOT x) → 1.
                Propagar la constante resultante.
must_not_change Comportamiento.
required_tests  Suite completa.
equivalence_risk low.
category        B — Leyes booleanas
introduced_in   1.0.0
notes           Caso fácil de pasar por alto si NOT x aparece a través
                de varios hops. El detector debe normalizar antes.
```

---

# Categoría C — De Morgan y empuje de NOT

Estas transformaciones reordenan la negación; aplicarlas solo cuando reducen una métrica.

## C.1 — `de_morgan_push`

```
id              de_morgan_push
name            Aplicación de De Morgan (push)
goal            ↓ gate_count o ↓ depth, según topología
preconditions   Existe NOT(AND(a, b)) o NOT(OR(a, b)).
can_change      NOT(AND(a, b)) → OR(NOT a, NOT b).
                NOT(OR(a, b))  → AND(NOT a, NOT b).
                Solo aplicar si reduce métricas (la transformación no es
                obligatoria; el optimizador la propone y la evalúa).
must_not_change Comportamiento sobre cualquier entrada.
                Etiquetas semánticas de las señales originales si están
                expuestas externamente.
required_tests  Suite completa.
equivalence_risk low en abstracto; medium en fan-out porque al insertar
                NOT(a) y NOT(b) puede aumentar fan-in en otras zonas.
                El evaluador valida si la métrica mejora.
category        C — De Morgan
introduced_in   1.0.0
notes           Tras aplicar C.1, suelen aparecer oportunidades para A.3
                (doble negación) y A.2 (compuertas redundantes).
```

## C.2 — `de_morgan_pull`

```
id              de_morgan_pull
name            Aplicación inversa de De Morgan (pull)
goal            ↓ gate_count cuando la forma "OR de NOTs" se vuelve más
                compacta como "NOT de AND" (o viceversa)
preconditions   Existe OR(NOT a, NOT b) o AND(NOT a, NOT b).
can_change      OR(NOT a, NOT b)  → NOT(AND(a, b)).
                AND(NOT a, NOT b) → NOT(OR(a, b)).
must_not_change Comportamiento.
required_tests  Suite completa.
equivalence_risk low en abstracto; medium en fan-out.
category        C — De Morgan
introduced_in   1.0.0
notes           Inverso de C.1. Solo aplicar si reduce métricas; típica-
                mente útil tras agrupar reducciones.
```

## C.3 — `not_propagation`

```
id              not_propagation
name            Propagación de NOT hacia la frontera
goal            ↓ depth en ciertas topologías; preparación para CSE
preconditions   Existe NOT en una posición interna donde empujarlo hacia
                las entradas (vía De Morgan repetido) reduce profundidad.
can_change      Reorganización del subgrafo aplicando C.1 / C.2
                iterativamente.
must_not_change Comportamiento.
required_tests  Suite completa.
equivalence_risk medium — la reorganización puede aumentar conteo si
                se aplica sin criterio.
category        C — De Morgan
introduced_in   1.0.0
notes           No es una transformación atómica sino una estrategia
                compuesta. Solo aplicar bajo guía métrica (mejora estricta).
```

---

# Categoría D — Distributividad

## D.1 — `distribute_and_over_or`

```
id              distribute_and_over_or
name            Distributividad de AND sobre OR
goal            ↓ depth o exposición de subexpresiones comunes (CSE).
preconditions   Existe AND(x, OR(y, z)).
can_change      AND(x, OR(y, z)) → OR(AND(x, y), AND(x, z)).
                Solo aplicar si:
                  - reduce depth, o
                  - genera subexpresión común con otra parte del grafo
                    aprovechable por CSE (E.1).
must_not_change Comportamiento.
required_tests  Suite completa.
equivalence_risk medium — duplica x; aumenta conteo si no compensa.
category        D — Distributividad
introduced_in   1.0.0
notes           Aplicar solo guiado por evaluación de mejora estricta
                en el conjunto de métricas.
```

## D.2 — `factor_and_out_of_or`

```
id              factor_and_out_of_or
name            Factorización: extracción de AND común
goal            ↓ gate_count
preconditions   Existe patrón OR(AND(x, y), AND(x, z)).
can_change      OR(AND(x, y), AND(x, z)) → AND(x, OR(y, z)).
must_not_change Comportamiento.
                Otros consumidores de las señales originales si los hay.
required_tests  Suite completa.
equivalence_risk low.
category        D — Distributividad
introduced_in   1.0.0
notes           Inverso de D.1; reduce conteo cuando se aplica sobre
                un patrón explícito de factorización común.
```

## D.3 — `distribute_or_over_and`

```
id              distribute_or_over_and
name            Distributividad de OR sobre AND
goal            preparación para CSE u otras reducciones
preconditions   Existe OR(x, AND(y, z)).
can_change      OR(x, AND(y, z)) → AND(OR(x, y), OR(x, z)).
must_not_change Comportamiento.
required_tests  Suite completa.
equivalence_risk medium — duplica x; aplicar solo cuando aporta.
category        D — Distributividad
introduced_in   1.0.0
```

## D.4 — `factor_or_out_of_and`

```
id              factor_or_out_of_and
name            Factorización: extracción de OR común
goal            ↓ gate_count
preconditions   Existe AND(OR(x, y), OR(x, z)).
can_change      AND(OR(x, y), OR(x, z)) → OR(x, AND(y, z)).
must_not_change Comportamiento.
required_tests  Suite completa.
equivalence_risk low.
category        D — Distributividad
introduced_in   1.0.0
```

---

# Categoría E — Subexpresiones comunes y reutilización

## E.1 — `common_subexpression_elimination`

```
id              common_subexpression_elimination
name            Detección de subexpresiones comunes (CSE)
goal            ↓ gate_count, ↑ reuse_score
preconditions   Existen dos subgrafos S1 y S2 tales que producen la misma
                función booleana sobre las mismas entradas (estructural-
                mente o tras normalización por A.2, B.x, C.x).
can_change      - Sustituir S2 por una referencia a la salida de S1.
                - Eliminar los nodos exclusivos de S2 mediante A.1.
                - Aumentar el fan-out de la salida de S1.
must_not_change Comportamiento.
                Etiquetas semánticas relevantes (si una de las dos copias
                tenía una etiqueta semántica única, decidir qué etiqueta
                conservar; en caso de conflicto la transformación se aplaza).
required_tests  Suite completa.
equivalence_risk medium — la detección de equivalencia requiere
                normalización; falsos positivos en detección rompen
                comportamiento (cubierto por la verificación obligatoria
                post-aplicación).
category        E — Reutilización
introduced_in   1.0.0
notes           Es una de las transformaciones de mayor impacto en
                circuitos multi-salida (half_adder, comparators,
                decoders).
                La detección recomendada es por hash topológico
                normalizado de subgrafos.
```

## E.2 — `output_sharing`

```
id              output_sharing
name            Compartición entre salidas
goal            ↓ gate_count
preconditions   Dos puertos de salida comparten una expresión que es
                idéntica o reducible a una señal común.
can_change      Crear una única señal interna compartida y conectar ambas
                salidas a ella (o a una variante mínima).
must_not_change Comportamiento.
                Identidad y etiquetas semánticas de los puertos de salida.
required_tests  Suite completa + verificación por señal semántica de
                cada salida implicada.
equivalence_risk low si la equivalencia se prueba por CSE; medium si se
                aproxima por inferencia parcial.
category        E — Reutilización
introduced_in   1.0.0
notes           Particularmente útil en comparadores y en decoders.
```

---

# Categoría F — Reducción de profundidad

## F.1 — `narrow_to_balanced_tree`

```
id              narrow_to_balanced_tree
name            Re-asociación de AND/OR n-ario a árbol balanceado
goal            ↓ depth
preconditions   Existe una cadena lineal de N AND consecutivos o N OR
                consecutivos asociados como ((((a ∧ b) ∧ c) ∧ d) ...).
can_change      Reorganizar como árbol balanceado de profundidad ⌈log2 N⌉.
must_not_change Comportamiento (AND/OR son asociativas).
                Conteo total de compuertas (no debe aumentar).
                Etiquetas semánticas que pudieran asignarse a la cadena
                "left" — solo se reorganiza topológicamente.
required_tests  Suite completa.
equivalence_risk low — asociatividad clásica.
category        F — Profundidad
introduced_in   1.0.0
notes           Importante en niveles ≥ 7 (sumadores ripple-carry).
```

## F.2 — `tree_to_narrow_for_reuse`

```
id              tree_to_narrow_for_reuse
name            Re-asociación de árbol a forma narrow para exponer reuso
goal            facilitar CSE posterior; potencialmente ↓ gate_count global
preconditions   Subgrafo en árbol balanceado donde la forma "narrow"
                expone una subexpresión común con otra parte del grafo.
can_change      Reorganización topológica preservando asociatividad.
must_not_change Comportamiento.
required_tests  Suite completa.
equivalence_risk medium — puede temporalmente aumentar profundidad.
                Solo aplicar si la mejora global tras E.1 lo compensa.
category        F — Profundidad
introduced_in   1.0.0
```

## F.3 — `hoist_common_subterm`

```
id              hoist_common_subterm
name            Hoisting de subexpresión común al nivel superior
goal            ↓ depth localmente, exponer reuso
preconditions   Subexpresión repetida en distintas ramas que puede
                computarse una sola vez "más cerca" de las entradas.
can_change      Crear señal intermedia única; conectar ramas a ella.
must_not_change Comportamiento.
required_tests  Suite completa.
equivalence_risk medium.
category        F — Profundidad
introduced_in   1.0.0
notes           Combina bien con E.1 (CSE).
```

---

# Categoría G — Pliegue de constantes (reformulada para Fase 1)

> **Nota normativa de Fase 1.** AONIX no admite constantes (`0`, `1`)
> como `SignalReference` primitiva (ver [03 — Formato `.aoncir`](03-format-aoncir.md)
> y [21 — Sintaxis física](21-aoncir-syntax.md)). Por tanto, las
> transformaciones de esta categoría **no introducen nodos constantes**;
> reorganizan el grafo cuando un subgrafo es **demostrablemente equivalente**
> a un valor constante. La "constante" es propiedad emergente del análisis,
> no entidad del modelo.

## G.1 — `constant_equivalent_subgraph_reduction`

```
id              constant_equivalent_subgraph_reduction
name            Reducción de subgrafo equivalente a constante
goal            ↓ gate_count
preconditions   Subgrafo cuyo valor de salida es demostrablemente
                constante para toda entrada del dominio (típicamente
                detectado tras aplicar B.3 aniquilación o B.5 complemento;
                ejemplo: AND(x, NOT x) ≡ 0; OR(x, NOT x) ≡ 1).
can_change      Redirigir consumidores del subgrafo hacia una rama del
                circuito que ya produzca ese valor por complemento
                (típicamente reutilizando una equivalencia disponible
                en el cono lógico de la misma salida).
                Si el valor constante solo se necesitaba para alimentar
                otra rama, y la rama puede simplificarse via B.x, hacer
                esa simplificación.
must_not_change Comportamiento sobre cualquier entrada.
                Identidad y semántica de puertos de salida del circuito.
required_tests  Suite completa.
equivalence_risk low (la equivalencia constante se prueba algebraicamente).
category        G — Constantes
introduced_in   1.0.0
notes           Nunca introduce un nodo "constante" como SignalReference
                en el `.aoncir`. Si el resultado de la transformación es
                que un puerto de salida del circuito debe valer siempre 0
                o 1, el circuito sigue conteniendo las compuertas que
                producen ese valor (por ejemplo y = a AND NOT a). El
                catálogo de tareas `constant_zero` y `constant_one` del
                nivel 1 exige justamente esto.
```

## G.2 — `constant_equivalent_propagation`

```
id              constant_equivalent_propagation
name            Propagación de subexpresión equivalente a constante
goal            ↓ gate_count, exposición de oportunidades B.x
preconditions   Una señal interna es demostrablemente equivalente a un
                valor constante sobre todo el dominio (resultado de
                análisis algebraico, no de fijación de puertos: AONIX no
                "fija" entradas externas).
can_change      Donde la señal alimente una compuerta cuya salida quede,
                tras la sustitución algebraica, simplificable por B.2/B.3,
                aplicar la simplificación.
must_not_change Comportamiento.
                Compuertas del grafo no dominadas por la equivalencia.
                Puertos del circuito.
required_tests  Suite completa.
equivalence_risk low si el análisis de constancia es correcto.
category        G — Constantes
introduced_in   1.0.0
notes           Igual que G.1: no introduce nodos constantes. Reformula
                regiones del grafo.
```

---

# Categoría H — Limpieza estructural

## H.1 — `gate_arity_collapse` (DESHABILITADA en Fase 1)

```
id              gate_arity_collapse
name            Colapso de aridad en cadenas asociativas
status          DISABLED in Phase 1
goal            (hipotético) ↓ gate_count, ↓ depth
preconditions   Existe AND(AND(a, b), c) o forma equivalente con OR.
can_change      (no aplicable) En Fase 1 AONIX usa aridad estricta
                binaria para AND y OR (ver docs/01 §R2). No existe la
                representación n-aria como destino legal de la
                transformación; intentar producir AND(a, b, c) sería
                un error de aridad rechazado por `Gate::new`.
must_not_change Aridad binaria estricta.
required_tests  N/A.
equivalence_risk N/A.
category        H — Limpieza
introduced_in   1.0.0 (catalogada para preservar coherencia con la
                visión a largo plazo)
notes           En Fase 1, la dirección útil es la opuesta: F.1
                `narrow_to_balanced_tree` ya organiza cadenas lineales
                en árboles binarios balanceados; eso se hace **sin
                cambiar la aridad** de cada gate individual. Una
                eventual extensión a primitivas n-arias en una
                `format_version` superior requeriría un cambio
                auditado de R2 (severidad S0 en docs/25: no auditable),
                por lo que H.1 permanece deshabilitada indefinidamente
                en la práctica.
```

## H.2 — `signal_renaming_normalization`

```
id              signal_renaming_normalization
name            Normalización de nombres de señales
goal            ninguna métrica funcional; mejora legibilidad y estabilidad
                del hash canónico bajo ordenamiento topológico
preconditions   Existen nombres de señal arbitrarios del agente.
can_change      Asignar nombres canónicos según convención
                (`s<n>` por orden topológico, conservando etiquetas
                semánticas y nombres de puertos).
must_not_change Comportamiento.
                Nombres de puertos (se preservan).
                Etiquetas semánticas asociadas.
                Hash canónico (cuidado: si el hash se calcula sobre
                nombres, la renormalización lo cambiaría; el hash
                canónico se define para ser invariante a renombrado;
                esta transformación produce el mismo hash).
required_tests  Suite completa (paranoia).
equivalence_risk low.
category        H — Limpieza
introduced_in   1.0.0
notes           Es maquillaje. No se cuenta como mejora estructural por
                el evaluador; el ranking_order la considera neutra.
```

---

# Categoría I — Transformaciones específicas de niveles temporales (visión futura)

Solo para niveles ≥ 11. **No bloquean Fase 1 ni siguientes inmediatas.** Se documentan para preservar coherencia con el roadmap.

## I.1 — `combinational_subgraph_optimization_in_temporal`

```
id              combinational_subgraph_optimization_in_temporal
name            Optimización de subgrafos combinacionales dentro de
                circuitos temporales
goal            ↓ gate_count, ↓ depth en la parte combinacional
preconditions   Subgrafo puramente combinacional (sin clock/reset/enable
                en su cono) dentro de un circuito que sí tiene parte
                temporal.
can_change      Aplicar transformaciones A.x–H.x al subgrafo combinacional.
must_not_change Estructura del bucle temporal.
                Señales etiquetadas clock, reset, enable.
                Comportamiento sobre todas las secuencias temporales
                verificadas.
required_tests  Suite completa (temporal incluida).
equivalence_risk medium — riesgo de tocar accidentalmente señales que
                interactúan con el bucle.
category        I — Temporal
introduced_in   1.0.0 (planeada)
notes           Documentada ahora para tenerla referenciada cuando
                lleguen niveles 11+.
```

---

# Transformaciones **prohibidas** (no es catálogo, es lista negra)

Estas transformaciones **no existen** en AONIX bajo ninguna versión del catálogo:

```
PROHIBITED — no se incluye en ninguna versión del catálogo

P.1   replace_xor_pattern_with_xor_gate
      "Detectar (a AND NOT b) OR (NOT a AND b) y sustituir por nodo XOR".
      JAMÁS. No existe nodo XOR. Crear esta transformación violaría R2.

P.2   replace_nand_pattern_with_nand_gate
      Idem para NAND.

P.3   replace_nor_pattern_with_nor_gate
      Idem para NOR.

P.4   replace_xnor_pattern_with_xnor_gate
      Idem para XNOR.

P.5   collapse_subcircuit_into_opaque_canonical_node
      "Detectar instancia interna de full_adder y reemplazarla por una
       'compuerta' full_adder opaca." JAMÁS. La versión oficial activa
      se serializa completamente expandida a AND/OR/NOT.
      La jerarquía es organización mental, no representación canónica.

P.6   skip_verification_to_save_time
      "Aplicar transformación y no re-verificar." JAMÁS.
      Toda transformación dispara re-verificación.

P.7   merge_xor_behavior_into_imported_macro
      Cualquier intento de "macro" que ofrezca un comportamiento derivado
      como bloque opaco invocable.
```

Estas prohibiciones se hacen cumplir por:

1. El **validador de acciones** rechaza acciones que pretendan introducir nodos prohibidos.
2. El **parser estricto del `.aoncir`** rechaza nodos con `kind` distinto a AND/OR/NOT.
3. El **propio catálogo de transformaciones** no contiene estas entradas (no son ejecutables).
4. Los **tests del propio AONIX** verifican que el optimizador no produce salidas que violen R2.

---

## Resumen del catálogo inicial

| ID | Categoría | Goal principal | Riesgo |
|----|-----------|----------------|--------|
| A.1 dead_signal_elimination | A | ↓ dead_signals | low |
| A.2 redundant_gate_elimination | A | ↓ gate_count | low |
| A.3 double_negation_elimination | A | ↓ gate_count, ↓ depth | low |
| B.1 idempotence | B | ↓ gate_count | low |
| B.2 identity | B | ↓ gate_count | low |
| B.3 annihilation | B | ↓ gate_count | low |
| B.4 absorption | B | ↓ gate_count, ↓ depth | low |
| B.5 complement | B | ↓ gate_count | low |
| C.1 de_morgan_push | C | ↓ métricas según topología | low/medium |
| C.2 de_morgan_pull | C | ↓ gate_count | low/medium |
| C.3 not_propagation | C | ↓ depth | medium |
| D.1 distribute_and_over_or | D | preparar CSE | medium |
| D.2 factor_and_out_of_or | D | ↓ gate_count | low |
| D.3 distribute_or_over_and | D | preparar CSE | medium |
| D.4 factor_or_out_of_and | D | ↓ gate_count | low |
| E.1 common_subexpression_elimination | E | ↓ gate_count, ↑ reuse_score | medium |
| E.2 output_sharing | E | ↓ gate_count | low/medium |
| F.1 narrow_to_balanced_tree | F | ↓ depth | low |
| F.2 tree_to_narrow_for_reuse | F | exponer reuso global | medium |
| F.3 hoist_common_subterm | F | ↓ depth, exponer reuso | medium |
| G.1 constant_equivalent_subgraph_reduction | G | ↓ gate_count | low |
| G.2 constant_equivalent_propagation | G | ↓ gate_count | low |
| H.1 gate_arity_collapse (disabled) | H | N/A en Fase 1 | N/A |
| H.2 signal_renaming_normalization | H | legibilidad | low (neutra) |
| I.1 combinational_subgraph_optimization_in_temporal | I | ↓ métricas en parte comb. | medium |

**Total inicial: 25 transformaciones**, distribuidas en 9 categorías. Catálogo abierto: cada incorporación futura incrementa la versión del catálogo y se documenta aquí.

## Pipeline operativo de aplicación (referencia)

Ver [15 — Reglas de optimización §"Pipeline de optimización"](15-optimization-rules.md) para el flujo completo. Resumen aplicado a este catálogo:

```
Orden recomendado (no obligatorio; el optimizador puede reordenar
si la mejora estricta lo justifica):

1. Constantes y aniquilaciones:   B.2 B.3 G.1 G.2
2. Eliminaciones triviales:        A.1 A.2 A.3 B.1 B.5
3. Absorción:                      B.4
4. Factorización:                  D.2 D.4
5. CSE y compartición:             E.1 E.2
6. De Morgan dirigido por métrica: C.1 C.2 C.3
7. Reducción de profundidad:       F.1 F.2 F.3
8. Limpieza estructural:           H.1 H.2

Tras cada transformación: re-verificación COMPLETA con la suite de la
tarea. Si falla: descartar la transformación y registrar la causa.

Iterar hasta punto fijo (ninguna transformación produce mejora
estricta) o hasta tope de iteraciones de la tarea.
```

## Garantías globales del catálogo

1. **Cierre bajo primitivas.** Ninguna transformación introduce un nodo de tipo distinto a AND, OR, NOT.
2. **Re-verificación obligatoria.** Toda transformación dispara la suite completa de la tarea.
3. **Memoria de optimización.** Toda aplicación o descarte queda registrada con delta y causa.
4. **Reversibilidad operativa.** Si una transformación se descarta tras la re-verificación, el estado anterior se conserva.
5. **Determinismo.** Mismo input + mismo catálogo + mismo orden de aplicación + misma semilla (si la hubiera) ⇒ mismo resultado.

## Decisiones cerradas en este documento

- Catálogo inicial de 25 transformaciones organizadas en 9 categorías.
- Política de re-verificación post-transformación.
- Lista negra explícita de transformaciones prohibidas.

## Decisiones que siguen abiertas

- Implementación concreta de cada detección de patrón (Fase 6 del roadmap).
- Algoritmo de detección de subgrafos equivalentes para E.1 (hash topológico vs SAT vs heurística).
- Heurísticas para decidir cuándo aplicar transformaciones medium-risk (C.x, D.1/D.3, F.2, F.3).
- Política exacta de tope de iteraciones por tarea.
- Política exacta de tolerancia epsilon para "mejora estricta".
- Catálogo de transformaciones temporales (niveles 11+) más allá de I.1.
