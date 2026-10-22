# NeoCruncher GUI 0.1.3: prueba en Ubuntu 24.04 x86_64

**0.1.3 elige Wayland nativo por defecto** cuando `XDG_SESSION_TYPE=wayland`
y `WAYLAND_DISPLAY` existe, incluso si GNOME también ofrece XWayland. Integra
la GUI Rust y el CLI de NeoCruncher 1.0 incluidos en el ensayo anterior. La
integración con AppImageLauncher aún requiere una prueba en el escritorio.

## AppImageLauncher y acceso desde el menú

Descarga `NeoCruncher-GUI-Prototype-0.1.3-Ubuntu24.04-x86_64.AppImage` y
`SHA256SUMS-0.1.3.txt` a la misma carpeta. Desde esa carpeta:

```sh
sha256sum -c SHA256SUMS-0.1.3.txt
chmod +x NeoCruncher-GUI-Prototype-0.1.3-Ubuntu24.04-x86_64.AppImage
```

Abre el AppImage y, cuando AppImageLauncher pregunte, elige **Integrate and
run**. AppImageLauncher normalmente mueve el archivo a su carpeta de
aplicaciones y crea el acceso del menú. Busca **NeoCruncher GUI (trial
0.1.3)**. Si conservas un AppImage anterior, abre la entrada que indica
**0.1.3**.

## Respaldo directo si AppImageLauncher sigue fallando

Descarga el `.tar.gz` 0.1.3 y su `SHA256SUMS-0.1.3-archive.txt`; comprueba
`sha256sum -c SHA256SUMS-0.1.3-archive.txt` y extráelo a una carpeta que no
moverás:

```sh
mkdir -p "$HOME/Applications/NeoCruncher-GUI-0.1.3"
tar -xzf NeoCruncher-GUI-Prototype-0.1.3-Ubuntu24.04-x86_64.tar.gz \
  -C "$HOME/Applications/NeoCruncher-GUI-0.1.3"
cd "$HOME/Applications/NeoCruncher-GUI-0.1.3"
./AppRun --diagnose
./AppRun --self-test
./AppRun
```

`--diagnose` debe mostrar `Selected display route: wayland`; `--self-test`
debe terminar con `bundled CPU CLI self-test passed`. Si abre correctamente,
ejecuta **una sola vez** `./install-desktop.sh` desde esa carpeta. El script
instala `~/.local/share/applications/neocruncher-gui-trial.desktop`, con el
icono que viene en el paquete, sin `sudo`. Después abre la GUI desde el menú
de aplicaciones; ya no necesitarás la terminal. No muevas ni borres la carpeta
extraída mientras uses ese acceso.

Para probar el cálculo, elige π, 1000 dígitos, CPU y verificación Full. Usa
una ruta de salida absoluta nueva dentro de una carpeta existente; espera
`Completed` y `PASS`. Si la ventana aún no abre, comparte la salida de
`./AppRun --diagnose`, del comando que falla y de `echo "$?"` ejecutado
inmediatamente después. `./AppRun --wayland` sigue disponible para forzar
el modo Wayland.

El [código fuente del frontend](../gui/README.md) y el
[código fuente del CLI 1.0](../Neocruncher-1.0.0-release.zip) están en este
repositorio. El paquete sigue siendo un ensayo de escritorio; los modos GPU
e híbrido no están cualificados aquí.
