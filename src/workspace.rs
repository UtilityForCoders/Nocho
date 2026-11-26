//! Botones para cambiar de página y acciones del panel Compartir, compartiendo geometría para
//! dibujo y clics.
//!
//! No hay cabecera, para que la isla ocupe poco alto: en la música, un botón de compartir a la
//! izquierda de los controles (simétrico al de la letra) abre Compartir; en Compartir, un botón
//! con una nota musical, junto al selector de dispositivo, vuelve a la música.
use crate::island::{self, Hit, LabelArea, NoticeIcon};
use tiny_skia::{PathBuilder, Pixmap};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Page {
    #[default]
    Music,
    Share,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShareAction {
    Files = 1,
    Clipboard = 2,
    Device,
    Inbox,
    Receiving,
}

#[derive(Default)]
pub struct ShareState {
    pub receiving: bool,
    pub device_name: String,
    pub busy: bool,
    pub status: String,
    pub unread: usize,
}

/// Margen lateral del contenido.
const PAD: f32 = 16.0;
/// Lado lógico del botón de compartir.
const PAGE_BUTTON: f32 = 30.0;
/// Centro horizontal lógico del botón de compartir: a la misma distancia del borde izquierdo
/// que el botón de la letra del derecho.
pub const SHARE_BUTTON_CX: f32 = 36.0;
/// Ancho lógico del botón para volver a la música, a la izquierda del selector de dispositivo.
const BACK_W: f32 = 34.0;
/// Línea del nivel de volumen, pegada al borde superior de la isla.
const VOLUME_Y: f32 = 6.0;

const DEVICE_Y: f32 = 14.0;
const DEVICE_H: f32 = 34.0;
const CARD_Y: f32 = DEVICE_Y + DEVICE_H + 8.0;
const CARD_H: f32 = 62.0;
const ROW_Y: f32 = CARD_Y + CARD_H + 8.0;
const ROW_H: f32 = 32.0;
const STATUS_Y: f32 = ROW_Y + ROW_H + 8.0;
pub const SHARE_HEIGHT: f32 = STATUS_Y + 24.0;

/// Rectángulo lógico del botón que lleva a `target`: el de compartir se ve en la música y el
/// de la música en Compartir. `height` = alto lógico de la isla: sin nada sonando la isla es
/// una sola fila y el botón queda centrado en ella; con reproductor, a la altura de los controles.
fn page_button(target: Page, height: f32) -> [f32; 4] {
    match target {
        Page::Share => {
            let cy = if height >= island::EXPANDED_COMPACT.1 * 0.75 {
                island::BUTTONS_Y
            } else {
                height / 2.0
            };
            [
                SHARE_BUTTON_CX - PAGE_BUTTON / 2.0,
                cy - PAGE_BUTTON / 2.0,
                PAGE_BUTTON,
                PAGE_BUTTON,
            ]
        }
        Page::Music => [PAD, DEVICE_Y, BACK_W, DEVICE_H],
    }
}

/// Centro lógico del botón que lleva a `target`, relativo a la esquina de la isla.
#[cfg(test)]
pub fn tab_center(target: Page, height: f32) -> (f32, f32) {
    let [x, y, w, h] = page_button(target, height);
    (x + w / 2.0, y + h / 2.0)
}

fn regions(width: f32, height: f32, page: Page) -> Vec<(Hit, [f32; 4])> {
    let target = match page {
        Page::Music => Page::Share,
        Page::Share => Page::Music,
    };
    let mut regions = vec![(Hit::Tab(target), page_button(target, height))];
    if page == Page::Share {
        let inner = width - 2.0 * PAD;
        let half = (inner - 8.0) / 2.0;
        let right = PAD + half + 8.0;
        let device_x = PAD + BACK_W + 6.0;
        regions.extend([
            (
                Hit::Share(ShareAction::Device),
                [device_x, DEVICE_Y, PAD + inner - device_x, DEVICE_H],
            ),
            (Hit::Share(ShareAction::Files), [PAD, CARD_Y, half, CARD_H]),
            (
                Hit::Share(ShareAction::Clipboard),
                [right, CARD_Y, half, CARD_H],
            ),
            (Hit::Share(ShareAction::Inbox), [PAD, ROW_Y, half, ROW_H]),
            (
                Hit::Share(ShareAction::Receiving),
                [right, ROW_Y, half, ROW_H],
            ),
        ]);
    }
    regions
}

pub fn hit_test(
    bounds: (f32, f32, f32, f32, f32),
    scale: f32,
    page: Page,
    x: f32,
    y: f32,
) -> Option<Hit> {
    let (left, top, width, height, _) = bounds;
    let (x, y) = ((x - left) / scale, (y - top) / scale);
    regions(width / scale, height / scale, page)
        .into_iter()
        .find(|(_, [rx, ry, w, h])| x >= *rx && x <= rx + w && y >= *ry && y <= ry + h)
        .map(|(hit, _)| hit)
}

/// Pinta en píxeles físicos a partir de medidas lógicas relativas a la esquina de la isla.
struct Canvas<'a> {
    pixmap: &'a mut Pixmap,
    left: f32,
    top: f32,
    scale: f32,
    alpha: f32,
}

impl Canvas<'_> {
    fn paint(&self, [r, g, b]: [u8; 3], opacity: f32) -> tiny_skia::Paint<'static> {
        let mut paint = tiny_skia::Paint {
            anti_alias: true,
            ..Default::default()
        };
        paint.set_color_rgba8(r, g, b, (opacity * self.alpha).clamp(0.0, 255.0) as u8);
        paint
    }

    fn fill(&mut self, rgb: [u8; 3], opacity: f32, [x, y, w, h]: [f32; 4], radius: f32) {
        let paint = self.paint(rgb, opacity);
        let s = self.scale;
        island::fill_rounded_rect(
            self.pixmap,
            &paint,
            self.left + x * s,
            self.top + y * s,
            w * s,
            h * s,
            radius * s,
        );
    }

    fn outline(&mut self, rgb: [u8; 3], opacity: f32, [x, y, w, h]: [f32; 4], radius: f32) {
        let paint = self.paint(rgb, opacity);
        let s = self.scale;
        if let Some(path) = island::rounded_rect_path(
            self.left + x * s + 0.5,
            self.top + y * s + 0.5,
            (w * s - 1.0).max(1.0),
            (h * s - 1.0).max(1.0),
            (radius * s - 0.5).max(0.0),
        ) {
            island::stroke_path(self.pixmap, &paint, &path, s.max(1.0));
        }
    }

    fn icon(&mut self, icon: NoticeIcon, rgb: [u8; 3], cx: f32, cy: f32, size: f32) {
        let paint = self.paint(rgb, 255.0);
        let s = self.scale;
        island::draw_notice_icon(
            self.pixmap,
            &paint,
            icon,
            self.left + cx * s,
            self.top + cy * s,
            s * size,
        );
    }

    /// Flecha hacia abajo del selector de dispositivo.
    fn chevron(&mut self, rgb: [u8; 3], cx: f32, cy: f32) {
        let paint = self.paint(rgb, 220.0);
        let s = self.scale;
        let (x, y) = (self.left + cx * s, self.top + cy * s);
        let mut pb = PathBuilder::new();
        pb.move_to(x - 4.5 * s, y - 2.2 * s);
        pb.line_to(x, y + 2.3 * s);
        pb.line_to(x + 4.5 * s, y - 2.2 * s);
        island::stroke(self.pixmap, &paint, pb, 1.6 * s);
    }

    #[allow(clippy::too_many_arguments)]
    fn label(
        &self,
        text: String,
        [x, y, w, h]: [f32; 4],
        alpha: f32,
        strong: bool,
        scroll: bool,
    ) -> LabelArea {
        let s = self.scale;
        LabelArea {
            text,
            x: self.left + x * s,
            y: self.top + y * s,
            w: (w * s).max(0.0),
            h: h * s,
            alpha: self.alpha * alpha,
            strong,
            scroll,
        }
    }
}

const WHITE: [u8; 3] = [255, 255, 255];

#[allow(clippy::too_many_arguments)]
pub fn draw(
    pixmap: &mut Pixmap,
    bounds: (f32, f32, f32, f32, f32),
    scale: f32,
    alpha: f32,
    page: Page,
    hovered: Option<Hit>,
    state: &ShareState,
    color: crate::settings::IslandColor,
) -> Vec<LabelArea> {
    let (left, top, width, height, _) = bounds;
    let (width, height) = (width / scale, height / scale);
    let accent = color.light();
    let mut canvas = Canvas {
        pixmap,
        left,
        top,
        scale,
        alpha,
    };
    let mut labels = Vec::new();

    for (hit, rect) in regions(width, height, page) {
        let hover = hovered == Some(hit);
        let [x, y, w, h] = rect;
        match hit {
            Hit::Tab(target) => {
                let icon = match target {
                    Page::Share => {
                        // Como los botones del reproductor: solo un círculo tenue bajo el mouse.
                        if hover {
                            canvas.fill(WHITE, 35.0, rect, w / 2.0);
                        }
                        NoticeIcon::Share
                    }
                    Page::Music => {
                        // Con el mismo estilo que el selector de dispositivo, a su lado.
                        canvas.fill(WHITE, if hover { 30.0 } else { 18.0 }, rect, 12.0);
                        canvas.outline(WHITE, 20.0, rect, 12.0);
                        NoticeIcon::Music
                    }
                };
                canvas.icon(icon, accent, x + w / 2.0, y + h / 2.0, 0.8);
            }
            Hit::Share(ShareAction::Device) => {
                canvas.fill(WHITE, if hover { 30.0 } else { 18.0 }, rect, 12.0);
                canvas.outline(WHITE, 20.0, rect, 12.0);
                canvas.icon(NoticeIcon::Wireless, WHITE, x + 22.0, y + h / 2.0, 0.72);
                let text = if !state.status.is_empty() {
                    state.status.clone()
                } else if state.busy {
                    "Solicitud en curso…".to_string()
                } else if state.device_name.is_empty() {
                    "Para: vincula un dispositivo…".to_string()
                } else {
                    format!("Para: {}", state.device_name)
                };
                labels.push(canvas.label(
                    text,
                    [x + 42.0, y + (h - 18.0) / 2.0, w - 72.0, 18.0],
                    1.0,
                    false,
                    true,
                ));
                canvas.chevron(WHITE, x + w - 18.0, y + h / 2.0);
            }
            Hit::Share(action @ (ShareAction::Files | ShareAction::Clipboard)) => {
                // Elegir archivos es la acción principal: va teñida con el color de la isla.
                let primary = action == ShareAction::Files;
                let (fill, border) = if primary {
                    (accent, if hover { 150.0 } else { 105.0 })
                } else {
                    (WHITE, if hover { 46.0 } else { 26.0 })
                };
                canvas.fill(
                    fill,
                    match (primary, hover) {
                        (true, true) => 62.0,
                        (true, false) => 44.0,
                        (false, true) => 30.0,
                        (false, false) => 18.0,
                    },
                    rect,
                    14.0,
                );
                canvas.outline(if primary { accent } else { WHITE }, border, rect, 14.0);
                let icon_box = [x + 12.0, y + (h - 34.0) / 2.0, 34.0, 34.0];
                canvas.fill(if primary { accent } else { WHITE }, 46.0, icon_box, 10.0);
                canvas.outline(WHITE, 30.0, icon_box, 10.0);
                let (icon, first, second) = match action {
                    ShareAction::Files => (NoticeIcon::Usb, "Elegir", "archivos"),
                    _ => (NoticeIcon::Keyboard, "Enviar", "portapapeles"),
                };
                canvas.icon(icon, WHITE, x + 29.0, y + h / 2.0, 0.72);
                let text_x = x + 54.0;
                let text_w = w - 58.0;
                labels.push(canvas.label(
                    first.to_string(),
                    [text_x, y + h / 2.0 - 18.0, text_w, 18.0],
                    1.0,
                    true,
                    false,
                ));
                labels.push(canvas.label(
                    second.to_string(),
                    [text_x, y + h / 2.0, text_w, 18.0],
                    1.0,
                    true,
                    true,
                ));
            }
            Hit::Share(ShareAction::Inbox) => {
                canvas.fill(WHITE, if hover { 30.0 } else { 16.0 }, rect, 12.0);
                canvas.outline(WHITE, 20.0, rect, 12.0);
                canvas.icon(NoticeIcon::Share, WHITE, x + 18.0, y + h / 2.0, 0.66);
                let text = match state.unread {
                    0 => "Mensajes recibidos".to_string(),
                    1 => "1 mensaje sin leer".to_string(),
                    n => format!("{n} mensajes sin leer"),
                };
                if state.unread > 0 {
                    canvas.fill(accent, 255.0, [x + 25.0, y + 7.0, 6.0, 6.0], 3.0);
                }
                labels.push(canvas.label(
                    text,
                    [x + 34.0, y + (h - 18.0) / 2.0, w - 40.0, 18.0],
                    1.0,
                    false,
                    true,
                ));
            }
            Hit::Share(ShareAction::Receiving) => {
                let on = state.receiving;
                canvas.fill(WHITE, if hover { 30.0 } else { 16.0 }, rect, 12.0);
                canvas.outline(
                    if on { accent } else { WHITE },
                    if on { 120.0 } else { 20.0 },
                    rect,
                    12.0,
                );
                canvas.icon(NoticeIcon::Wireless, WHITE, x + 18.0, y + h / 2.0, 0.66);
                let text = if on {
                    "Recepción activa"
                } else {
                    "No molestar"
                };
                if on {
                    canvas.fill(
                        accent,
                        255.0,
                        [x + w - 18.0, y + h / 2.0 - 4.0, 8.0, 8.0],
                        4.0,
                    );
                }
                labels.push(canvas.label(
                    text.to_string(),
                    [x + 34.0, y + (h - 18.0) / 2.0, w - 58.0, 18.0],
                    if on { 1.0 } else { 0.7 },
                    false,
                    true,
                ));
            }
            _ => {}
        }
    }
    if page == Page::Share {
        let text = if state.receiving {
            "Tus equipos vinculados pueden enviarte archivos"
        } else {
            "Recepción pausada: no se aceptan envíos"
        };
        labels.push(canvas.label(
            text.to_string(),
            [PAD + 4.0, STATUS_Y, width - 2.0 * PAD - 8.0, 16.0],
            0.55,
            false,
            true,
        ));
    }
    labels
}

/// Nivel de volumen en una línea fina junto al borde superior de la isla, mientras se cambia.
pub fn draw_volume(
    pixmap: &mut Pixmap,
    bounds: (f32, f32, f32, f32, f32),
    scale: f32,
    alpha: f32,
    level: f32,
    color: crate::settings::IslandColor,
) {
    let (left, top, width, _, _) = bounds;
    let mut canvas = Canvas {
        pixmap,
        left,
        top,
        scale,
        alpha,
    };
    let w = width / scale - 2.0 * PAD;
    let y = VOLUME_Y;
    canvas.fill(WHITE, 55.0, [PAD, y, w, 3.0], 1.5);
    canvas.fill(
        color.light(),
        255.0,
        [PAD, y, w * level.clamp(0.0, 1.0), 3.0],
        1.5,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn botones_y_acciones_coinciden_a_distintos_dpi() {
        for scale in [1.0, 1.25, 1.5, 2.0] {
            let width = island::EXPANDED_COMPACT.0;
            let bounds = (80.0, 8.0, width * scale, SHARE_HEIGHT * scale, 24.0 * scale);
            let regions = regions(width, SHARE_HEIGHT, Page::Share);
            for (index, (_, [x, y, w, h])) in regions.iter().enumerate() {
                assert!(*x >= 16.0 && x + w <= width - 16.0);
                assert!(*y >= 8.0 && y + h <= SHARE_HEIGHT - 12.0);
                for (_, [ox, oy, ow, oh]) in &regions[index + 1..] {
                    assert!(x + w <= *ox || ox + ow <= *x || y + h <= *oy || oy + oh <= *y);
                }
            }
            for (hit, [x, y, w, h]) in regions {
                assert!(
                    hit_test(
                        bounds,
                        scale,
                        Page::Share,
                        bounds.0 + (x + w / 2.0) * scale,
                        bounds.1 + (y + h / 2.0) * scale
                    ) == Some(hit)
                );
            }
            // En la música solo responde el botón de compartir: el resto es del reproductor.
            let music_h = island::EXPANDED_COMPACT.1;
            let music = (80.0, 8.0, width * scale, music_h * scale, 24.0 * scale);
            assert!(hit_test(
                music,
                scale,
                Page::Music,
                music.0 + 60.0 * scale,
                music.1 + 100.0 * scale
            )
            .is_none());
            // Alineado con los controles, o centrado en la isla de una sola fila sin música.
            for (height, expected_y) in [(music_h, island::BUTTONS_Y), (56.0, 28.0)] {
                let (sx, sy) = tab_center(Page::Share, height);
                assert_eq!((sx, sy), (SHARE_BUTTON_CX, expected_y));
                let at = (80.0, 8.0, width * scale, height * scale, 24.0 * scale);
                assert!(
                    hit_test(at, scale, Page::Music, at.0 + sx * scale, at.1 + sy * scale)
                        == Some(Hit::Tab(Page::Share))
                );
            }
            let mut image = Pixmap::new(640, 400).unwrap();
            draw_volume(
                &mut image,
                bounds,
                scale,
                1.0,
                0.5,
                crate::settings::IslandColor::default(),
            );
            let y = (bounds.1 + (VOLUME_Y + 1.5) * scale) as u32;
            let inner = width - 2.0 * PAD;
            for (x, expected) in [
                (PAD - 4.0, 0),
                (PAD + inner * 0.25, 255),
                (PAD + inner * 0.75, 55),
                (width - PAD + 4.0, 0),
            ] {
                let x = (bounds.0 + x * scale) as u32;
                assert_eq!(image.pixel(x, y).unwrap().alpha(), expected);
            }
        }
    }

    #[test]
    fn compartir_muestra_estado_y_bandeja_legibles() {
        let mut image = Pixmap::new(400, 260).unwrap();
        let width = island::EXPANDED_COMPACT.0;
        let labels = draw(
            &mut image,
            (0.0, 0.0, width, SHARE_HEIGHT, 24.0),
            1.0,
            1.0,
            Page::Share,
            None,
            &ShareState {
                receiving: true,
                unread: 2,
                device_name: "Portátil".into(),
                ..Default::default()
            },
            crate::settings::IslandColor::default(),
        );
        let has = |labels: &[LabelArea], text: &str| labels.iter().any(|label| label.text == text);
        assert!(has(&labels, "Recepción activa"));
        assert!(has(&labels, "2 mensajes sin leer"));
        assert!(has(&labels, "Para: Portátil"));
        assert!(has(&labels, "Elegir") && has(&labels, "archivos"));
        assert!(has(&labels, "Enviar") && has(&labels, "portapapeles"));
        for label in &labels {
            assert!(
                label.x >= 0.0 && label.x + label.w <= width,
                "{}",
                label.text
            );
            assert!(
                label.y >= 0.0 && label.y + label.h <= SHARE_HEIGHT,
                "{}",
                label.text
            );
        }

        let paused = draw(
            &mut image,
            (0.0, 0.0, width, SHARE_HEIGHT, 24.0),
            1.0,
            1.0,
            Page::Share,
            None,
            &ShareState {
                status: "Enviando a Portátil…".into(),
                ..Default::default()
            },
            crate::settings::IslandColor::default(),
        );
        assert!(has(&paused, "No molestar"));
        assert!(has(&paused, "Enviando a Portátil…"));
    }

    #[test]
    fn cada_pagina_muestra_el_boton_de_la_otra() {
        let width = island::EXPANDED_COMPACT.0;
        for (page, target, height) in [
            (Page::Music, Page::Share, island::EXPANDED_COMPACT.1),
            (Page::Share, Page::Music, SHARE_HEIGHT),
        ] {
            let mut image = Pixmap::new(400, 260).unwrap();
            let labels = draw(
                &mut image,
                (0.0, 0.0, width, height, 24.0),
                1.0,
                1.0,
                page,
                Some(Hit::Tab(target)),
                &ShareState::default(),
                crate::settings::IslandColor::default(),
            );
            // Sin textos de pestañas: el botón es solo un ícono.
            assert!(!labels
                .iter()
                .any(|l| l.text == "Música" || l.text == "Compartir"));
            let (cx, cy) = tab_center(target, height);
            let alpha = image.pixel((cx + 11.0) as u32, cy as u32).unwrap().alpha();
            assert!(alpha > 0, "{target:?} en {page:?}");
            // Nada de la cabecera de antes (pestañas, separador) arriba del contenido.
            for x in [60, 160, 260] {
                assert_eq!(image.pixel(x, 4).unwrap().alpha(), 0, "{page:?}");
            }
        }
    }
}
