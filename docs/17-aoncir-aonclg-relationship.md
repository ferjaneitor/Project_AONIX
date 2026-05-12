# 17 — Relación formal entre `.aoncir` y `.aonclg`

> **Documento normativo.** Fija qué representa cada formato, cómo se enlazan, qué garantías ofrecen separados y juntos, y por qué la separación es inviolable.

## Resumen en una línea

> `.aoncir` es **qué es** un circuito. `.aonclg` es **cómo se aprendió a llegar**.

## Identidades

| | `.aoncir` | `.aonclg` |
|---|-----------|-----------|
| Nombre completo | AON Circuit Intermediate Representation | AON Canonical Learning Graph |
| Rol | Verdad técnica canónica del circuito | Contexto de aprendizaje asociado |
| Determinado por | Estructura del grafo lógico | Trayectoria del episodio |
| Audiencia primaria | Simulador, verificador, evaluador, visualizador | Memoria de aprendizaje, entrenamiento de IA |
| Lectura humana | Sí (auditable) | Sí (revisable) |
| Inmutable | Por reemplazo atómico de versión oficial | Tras cerrar el episodio |
| Introduce primitivas | **No** (solo usa AND/OR/NOT) | **No** (no puede introducir ninguna primitiva) |
| Pasa por verificador | Sí, obligatorio para ser oficial | No es su objeto |
| Pasa por validador de acciones | Su construcción sí, su archivo no | Refleja decisiones del validador |
| Determina correctitud | El verificador la determina **sobre** `.aoncir` | El `.aonclg` la **registra**, no la decide |

## Diagrama de relación

```
                          ┌───────────────────┐
                          │      TAREA        │
                          │   (Task, doc 12)  │
                          └─────────┬─────────┘
                                    │
                  ┌─────────────────┼─────────────────┐
                  │                 │                 │
                  ▼                 ▼                 ▼
        ┌───────────────────┐                ┌───────────────────┐
        │  Memoria canónica │                │ Memoria aprend.   │
        │                   │                │                   │
        │  one_bit_full_    │   referenced   │  episode_42.aonclg│
        │  adder.aoncir     │◄──────by_hash──┤                   │
        │  (oficial activo) │                │                   │
        └─────────┬─────────┘                └───────────────────┘
                  │                                    ▲
                  │ reemplazo                          │ uno o varios
                  │ atómico                            │ .aonclg
                  ▼                                    │ por .aoncir
        ┌───────────────────┐                          │
        │ Memoria histórica │                          │
        │  versiones        │◄─────────────────────────┘
        │  previas verif.   │           (un .aonclg histórico
        └───────────────────┘            puede referenciar una
                                         versión histórica)
```

## Reglas formales de la relación

### R-AC.1 — Referencia siempre por hash canónico

Un `.aonclg` referencia un `.aoncir` **por su hash canónico**, no por nombre ni por ruta. Esto sobrevive a renombramientos, movimientos en el sistema de archivos y promociones.

### R-AC.2 — Existencia desacoplada en una dirección

- Un `.aoncir` puede existir **sin** `.aonclg` asociado. Ejemplo: circuitos producidos por búsqueda exhaustiva pura, importaciones manuales, o reescrituras puramente algebraicas no protagonizadas por un agente que registre trayectoria.
- Un `.aonclg` **no puede** existir sin referenciar al menos un `.aoncir`. Sin circuito, no hay aprendizaje sobre el cual narrar.

### R-AC.3 — Multiplicidad permitida

Un `.aoncir` puede tener **0 o muchos** `.aonclg` asociados:

- Cada episodio cerrado de un agente sobre esa tarea produce su propio `.aonclg`.
- Mismo agente, distintas tareas que produjeron el mismo `.aoncir` final, generan distintos `.aonclg`.
- Distintos agentes pueden generar distintos `.aonclg` que referencien el mismo `.aoncir` cuando sus episodios convergen en el mismo circuito.

### R-AC.4 — Cardinalidad de versiones canónicas

Para un circuito y tamaño dados existe **una sola versión oficial activa** (un solo `.aoncir` oficial). Sus versiones previas viven en memoria histórica. Detalle completo en [19 — Política de versionado](19-versioning-policy.md).

Los `.aonclg` históricos pueden referenciar:

- la versión oficial activa actual,
- una versión histórica concreta (por su hash canónico),
- o un circuito intermedio del propio episodio que nunca fue promovido.

### R-AC.5 — Inmutabilidad de un `.aonclg` cerrado

Una vez cerrado el episodio, el `.aonclg` es **inmutable**. Esto incluye:

- La trayectoria.
- Las decisiones del validador, verificador y evaluador citadas dentro.
- El hash del `.aoncir` referenciado (si la oficial activa cambia luego, el `.aonclg` sigue apuntando al hash que vio en su momento; no se actualiza automáticamente).
- Las recompensas calculadas.

Modificar un `.aonclg` cerrado equivale a falsificar historia.

### R-AC.6 — El `.aonclg` no puede modificar el `.aoncir`

Ninguna información contenida en un `.aonclg` puede mutar el grafo del `.aoncir` que referencia. Si un agente "descubre" una mejora durante el aprendizaje, la mejora produce un **nuevo circuito candidato**, pasa por verificación/evaluación, y puede convertirse en una nueva versión oficial activa — pero esa promoción **no se hace desde el `.aonclg`**, se hace por el coordinador en su flujo de cierre de episodio.

### R-AC.7 — Ninguno introduce primitivas

Ni `.aoncir` ni `.aonclg` pueden introducir nodos lógicos fuera de `{AND, OR, NOT}`. El `.aoncir` lo respeta en su grafo; el `.aonclg` lo respeta porque **no contiene grafo propio**: solo metadata, trayectoria y referencias.

### R-AC.8 — Promoción de circuito + actualización de aprendizaje

Cuando un episodio cierra con promoción de un nuevo oficial activo:

- El `.aoncir` oficial activo se reemplaza atómicamente (memoria canónica).
- La versión anterior pasa a memoria histórica.
- Se crea un `.aonclg` final del episodio con:
  - Hash del `.aoncir` final.
  - Hash del oficial activo previo (ahora histórico).
  - Bandera `promoted: true`.
  - Delta estructural y de métricas.

Si **no** hay promoción (el circuito es correcto pero no supera al oficial activo), el `.aonclg` se crea igualmente con `promoted: false` y la causa (empate, no-mejora, etc.).

### R-AC.9 — Referencias huérfanas

Si un `.aoncir` se elimina por una operación administrativa (raro, requiere auditoría), todos los `.aonclg` que lo referenciaban quedan **huérfanos**. AONIX los marca como huérfanos en memoria de aprendizaje pero no los borra: su trayectoria sigue siendo registro de aprendizaje. La eliminación de `.aoncir` se considera **operación excepcional** y debe quedar trazada.

### R-AC.10 — Aliasado prohibido

Un `.aonclg` no puede declarar como `.aoncir` referenciado un archivo cuyo hash canónico difiera del que tenía cuando se creó. No hay reescritura de referencias. Si el agente trabajó sobre un circuito intermedio, el `.aonclg` referencia ese circuito intermedio por su hash específico de aquel momento, no el "más actualizado".

## Información que vive en uno y no en el otro

| Información | `.aoncir` | `.aonclg` |
|------------|-----------|-----------|
| Topología del grafo | ✓ | — |
| Tipo de cada nodo | ✓ | — |
| Aridades, conexiones | ✓ | — |
| Etiquetas semánticas de señales | ✓ | (referenciadas) |
| Hash canónico | ✓ | (referenciado) |
| Predecesor canónico | ✓ | (referenciado) |
| Reporte de verificación de la versión | ✓ (sello) | (resumido como resultado del episodio) |
| Métricas del evaluador finales | ✓ | (resumido) |
| Layout 2D | ✓ (opcional) | — |
| Meta de la tarea | (referenciada) | ✓ |
| Estado inicial del episodio | — | ✓ |
| Trayectoria de acciones | — | ✓ |
| Retroalimentación del validador por acción | — | ✓ |
| Resultados de simulador durante el episodio | — | ✓ |
| Estado parcial intermedio del circuito | — | ✓ |
| Recompensas paso a paso | — | ✓ |
| Errores comunes detectados durante el episodio | — | ✓ |
| Suites de pruebas usadas | (referenciadas) | ✓ |
| Semilla aleatoria | — | ✓ |
| Identidad del agente | (creador en meta) | ✓ |
| Versiones de modelos y AONIX | — | ✓ |
| Comparación contra oficial activo | (predecesor) | ✓ |
| Bandera de promoción | — | ✓ |

## Casos límite y aclaraciones

### Caso 1: Episodio sin construcción final

Un episodio puede cerrar sin producir un `.aoncir` final (el agente abandona, los pasos se agotan, los rechazos L0 son persistentes). En ese caso:

- No hay `.aoncir` final del episodio.
- El `.aonclg` referencia el último circuito **válido** alcanzado (puede ser el inicial vacío).
- El `.aonclg` registra causa de cierre sin éxito.

### Caso 2: Múltiples `.aoncir` candidatos producidos en un episodio

Durante construcción interactiva, el grafo cambia. AONIX puede serializar snapshots intermedios (uso interno, no necesariamente en memoria canónica). El `.aonclg` puede referenciar varios snapshots intermedios por hash, pero solo **uno** como `.aoncir` final del episodio.

### Caso 3: Distintos episodios convergen al mismo `.aoncir`

Dos agentes resuelven la misma tarea y producen circuitos estructuralmente equivalentes (mismo hash canónico). Resultado:

- **Un solo `.aoncir`** en memoria canónica (oficial activo si gana, histórico si no).
- **Dos `.aonclg`** distintos en memoria de aprendizaje, ambos referenciando ese hash.
- Las trayectorias son distintas; el destino es el mismo.

### Caso 4: Reemplazo de oficial activo entre la creación y el cierre

Mientras un agente A construye, otro agente B promueve un nuevo oficial activo. El `.aonclg` de A registra el hash del oficial activo que A vio (si lo vio en modo estudio post-resolución previo). Si A logra superar al nuevo incumbente, su episodio promueve normalmente; si no, queda como solución no-promovida. El `.aonclg` registra la situación de carrera.

### Caso 5: Estudio de un `.aoncir` sin agente

Un investigador inspecciona memoria histórica. No hay episodio, no hay `.aonclg` nuevo. Solo lectura. Esto es operación legítima sobre los archivos canónicos sin afectar nada.

## Formato físico (decisiones pendientes)

Ambos formatos comparten decisiones pendientes (ver [11 — Roadmap](11-roadmap.md)):

1. Representación: texto legible, binario, o híbrido.
2. Codificación del hash canónico (función hash, longitud).
3. Forma de las referencias (URI propio, hash crudo).

Estas decisiones afectan a ambos formatos por igual; la relación entre ellos es independiente de la representación física.

## Errores comunes a evitar

1. **Fundir formatos** en uno solo "porque comparten metadata". Prohibido. La separación es la garantía de que el aprendizaje no contamina la verdad.
2. **Usar `.aonclg` como fuente para reemplazar `.aoncir`.** Prohibido. La promoción pasa por verificador y coordinador, no por aprendizaje.
3. **Re-actualizar el hash referenciado** en un `.aonclg` cerrado para "mantenerlo al día" tras una nueva promoción. Prohibido. El `.aonclg` apunta al hash que vio.
4. **Compartir un `.aonclg` con el agente activo** sobre la tarea cuyo `.aoncir` referencia, salvo modo estudio. Sería atajo.
5. **Permitir que un `.aonclg` introduzca etiquetas semánticas en el circuito** referenciado. El `.aonclg` puede contener notas semánticas sobre el episodio, pero no añade etiquetas al grafo del `.aoncir`.

## Compatibilidad de versiones

Cuando AONIX evoluciona y el formato `.aoncir` o `.aonclg` cambia de `format_version`:

- Los archivos antiguos se mantienen.
- Los parsers retrocompatibles los soportan hasta donde sea razonable.
- Las referencias por hash canónico **sobreviven** porque el hash se calcula sobre estructura abstracta, no sobre serialización física.

## Lo que la relación entre formatos **garantiza**

- Auditabilidad completa: dada cualquier versión canónica, se sabe quién la produjo, con qué trayectoria, en qué nivel, con qué semilla, en qué episodio.
- Reproducibilidad: cualquier episodio puede re-ejecutarse con el mismo agente, misma semilla, misma tarea-versión, mismo AONIX-versión, y produce la misma trayectoria.
- Separación de roles: el aprendizaje no decide verdad; la verdad no archiva aprendizaje innecesario.
- Independencia parcial: borrar todos los `.aonclg` de una familia no destruye los `.aoncir` ni cambia la verdad técnica. (Pierde aprendizaje, no verdad.)
