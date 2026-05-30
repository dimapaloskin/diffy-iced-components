mod effect;
mod gutter_draw;
mod scrollbars;

use std::sync::Arc;

use iced::Element;
use iced::Length;
use iced::advanced::graphics::text;
use iced::advanced::graphics::text::Renderer as TextRendererTrait;
use iced::advanced::layout;
use iced::advanced::renderer::Renderer as RendererTrait;
use iced::advanced::widget::{self, Tree, Widget};
use iced::mouse::ScrollDelta;
use iced::time::Instant;

use diffy_ui::Theme;

use crate::document::Document;
use crate::gutter::{GutterConfig, GutterMetricsRequest, GutterRenderArtifactRequest};
use crate::insets::Insets;
use crate::measurement::{MeasurementRequest, MeasurementResult};
use crate::scroll::ScrollMapInputs;
use crate::state::State;
use crate::style::Style;
use crate::text_layout::{TextLayoutConfig, TextLayoutRequest};
use crate::viewport::Viewport;

use self::effect::Effect;
use self::scrollbars::Scrollbars;

pub(crate) struct CodeView<'a, Message> {
  inputs: Inputs<'a>,
  on_measure_request: fn(MeasurementRequest) -> Message,
}

pub(crate) struct Inputs<'a> {
  pub(crate) document: &'a Document,
  pub(crate) width: Length,
  pub(crate) height: Length,
  pub(crate) text_layout_config: TextLayoutConfig,
  pub(crate) insets: Insets,
  pub(crate) border_radius: iced::border::Radius,
  pub(crate) gutter_config: GutterConfig,
  pub(crate) style: Style,
  pub(crate) measurement_result: Option<&'a MeasurementResult>,
}

impl<'a, Message> CodeView<'a, Message> {
  pub(crate) fn new(
    inputs: Inputs<'a>,
    on_measure_request: fn(MeasurementRequest) -> Message,
  ) -> Self {
    Self {
      inputs,
      on_measure_request,
    }
  }
}

impl<'a, Message, Renderer> Widget<Message, Theme, Renderer> for CodeView<'a, Message>
where
  Renderer: RendererTrait + TextRendererTrait,
{
  fn size(&self) -> iced::Size<iced::Length> {
    iced::Size::new(self.inputs.width, self.inputs.height)
  }

  fn tag(&self) -> iced::advanced::widget::tree::Tag {
    widget::tree::Tag::of::<State>()
  }

  fn state(&self) -> widget::tree::State {
    widget::tree::State::new(State::default())
  }

  fn update(
    &mut self,
    tree: &mut Tree,
    event: &iced::Event,
    layout: layout::Layout<'_>,
    cursor: iced::advanced::mouse::Cursor,
    _renderer: &Renderer,
    shell: &mut iced::advanced::Shell<'_, Message>,
    _viewport: &iced::Rectangle,
  ) {
    use iced::Event;
    use iced::mouse::{Button, Event as MouseEvent};
    use iced::window::Event::RedrawRequested;

    match event {
      Event::Window(RedrawRequested(now)) => {
        self.on_redraw_requested(tree, layout.bounds(), *now, shell);
      }
      Event::Mouse(MouseEvent::WheelScrolled { delta }) => {
        self.on_mouse_wheel(tree, delta, layout, cursor, shell);
      }
      Event::Mouse(MouseEvent::ButtonPressed(Button::Left)) => {
        Scrollbars::from_inputs(&self.inputs)
          .on_press(tree, layout.bounds(), cursor)
          .apply(shell);
      }
      Event::Mouse(MouseEvent::CursorMoved { .. }) => {
        Scrollbars::from_inputs(&self.inputs)
          .on_cursor_moved(tree, layout.bounds(), cursor)
          .apply(shell);
      }
      Event::Mouse(MouseEvent::CursorLeft) => {
        Scrollbars::from_inputs(&self.inputs)
          .on_cursor_left(tree, layout.bounds())
          .apply(shell);
      }
      Event::Mouse(MouseEvent::ButtonReleased(Button::Left)) => {
        Scrollbars::from_inputs(&self.inputs)
          .on_release(tree, layout.bounds(), cursor)
          .apply(shell);
      }
      _ => {}
    }
  }

  fn layout(
    &mut self,
    tree: &mut Tree,
    _renderer: &Renderer,
    limits: &iced::advanced::layout::Limits,
  ) -> layout::Node {
    let Inputs {
      document,
      width,
      height,
      insets,
      text_layout_config,
      gutter_config,
      measurement_result,
      ..
    } = self.inputs;

    let state = tree.state.downcast_mut::<State>();
    let resolved_size = limits.resolve(width, height, iced::Size::ZERO);
    let gutter_metrics = state.gutter.ensure_metrics(GutterMetricsRequest {
      document,
      text_layout_config,
      gutter_config,
    });
    let document_changed = state
      .text_layout
      .visible_layout()
      .is_some_and(|entry| entry.key.document_revision != self.inputs.document.revision());
    let scrollbars = Scrollbars::from_inputs(&self.inputs);

    let viewport = Viewport::new(
      resolved_size,
      insets,
      gutter_metrics,
      scrollbars.right_chrome_reserve(),
    );

    let measurement_request = MeasurementRequest::new(
      document,
      text_layout_config,
      viewport.text.content_bounds.size().width,
    );
    let measurement_key = measurement_request.key;

    let measurement_result =
      state.update_pending_measurement(measurement_request, measurement_result);

    let map_inputs = ScrollMapInputs {
      source_line_count: document.line_count(),
      mode: text_layout_config.wrap_mode,
      line_height: text_layout_config.line_height,
      measurement_key,
    };

    let scroll_viewport_size = viewport.scroll_viewport_size();
    if document_changed {
      state.scroll.reset(map_inputs);
    } else {
      state.scroll.reconcile(map_inputs, scroll_viewport_size);
    }

    if let Some(result) = measurement_result {
      state
        .scroll
        .apply_measurement_result(result, scroll_viewport_size);
    }

    let text_layout_request = TextLayoutRequest {
      document,
      content_size: viewport.text.content_bounds.size(),
      vertical_scroll: state.scroll.vertical(),
      config: text_layout_config,
    };

    let gutter_size = viewport
      .gutter
      .map(|area| area.content_bounds.size())
      .unwrap_or(iced::Size::ZERO);

    let visible_text_layout = state.text_layout.ensure_visible_layout(text_layout_request);
    state
      .scroll
      .sync_from_cosmic(visible_text_layout.cosmic_scroll);

    let gutter_render_artifact_request = GutterRenderArtifactRequest {
      text_layout_config,
      metrics: gutter_metrics,
      gutter_size,
      projection: &visible_text_layout.projection,
    };

    state
      .gutter
      .ensure_render_artifact(gutter_render_artifact_request);

    state.viewport = viewport;

    layout::Node::new(resolved_size)
  }

  fn draw(
    &self,
    tree: &Tree,
    renderer: &mut Renderer,
    theme: &Theme,
    _style: &iced::advanced::renderer::Style,
    layout: iced::advanced::Layout<'_>,
    _cursor: iced::advanced::mouse::Cursor,
    viewport: &iced::Rectangle,
  ) {
    use iced::advanced::renderer::Quad;

    let state = tree.state.downcast_ref::<State>();
    let bounds = layout.bounds();
    let Some(visible_bounds) = bounds.intersection(viewport) else {
      return;
    };

    let quad = Quad {
      bounds: visible_bounds,
      border: iced::Border {
        color: iced::Color::TRANSPARENT,
        width: 0.0,
        radius: self.inputs.border_radius,
      },
      ..Quad::default()
    };

    renderer.fill_quad(quad, self.inputs.style.background_color);

    self.draw_gutter(state, renderer, bounds, viewport);

    let text_content_bounds = state.viewport.absolute_text_content_bounds(bounds);
    if let (Some(entry), Some(clip_bounds)) = (
      state.text_layout.visible_layout(),
      text_content_bounds.intersection(viewport),
    ) {
      let position = iced::Point::new(
        text_content_bounds.x - state.scroll.horizontal_px(),
        text_content_bounds.y,
      );

      renderer.fill_raw(text::Raw {
        buffer: Arc::downgrade(entry.payload.buffer()),
        position,
        color: self.inputs.style.text_color,
        clip_bounds,
      });
    }

    let scrollbars = Scrollbars::from_inputs(&self.inputs);
    scrollbars.draw_vertical_overlay(state, renderer, theme, bounds);
  }
}

impl<'a, Message> CodeView<'a, Message> {
  fn scroll_delta_to_pixels(&self, delta: &ScrollDelta) -> iced::Vector {
    match delta {
      ScrollDelta::Pixels { x, y } => iced::Vector::new(*x, *y),
      ScrollDelta::Lines { x, y } => {
        // TODO: move magic number to const after reshaping file structure
        let step = self.inputs.text_layout_config.line_height * 3.0;
        iced::Vector::new(*x * step, *y * step)
      }
    }
  }

  fn on_mouse_wheel(
    &mut self,
    tree: &mut Tree,
    delta: &ScrollDelta,
    layout: layout::Layout<'_>,
    cursor: iced::advanced::mouse::Cursor,
    shell: &mut iced::advanced::Shell<'_, Message>,
  ) {
    // Handle wheel over the whole CodeView area.
    // If text-only behavior is needed later, narrow this to the content bounds
    if !cursor.is_over(layout.bounds()) {
      return;
    }

    let mut effect = Effect::capture_event();
    let delta = self.scroll_delta_to_pixels(delta);
    let state = tree.state.downcast_mut::<State>();
    let scroll_viewport = state.viewport.scroll_viewport_size();

    effect.merge(Effect::from_scroll_change(
      state.scroll.apply_wheel_delta(delta, scroll_viewport),
    ));

    effect.merge(Scrollbars::from_inputs(&self.inputs).note_activity(tree, Instant::now()));

    effect.apply(shell);
  }

  fn on_redraw_requested(
    &mut self,
    tree: &mut widget::Tree,
    _widget_bounds: iced::Rectangle,
    now: iced::time::Instant,
    shell: &mut iced::advanced::Shell<'_, Message>,
  ) {
    Scrollbars::from_inputs(&self.inputs)
      .on_redraw_requested(tree, now)
      .apply(shell);

    let state = tree.state.downcast_mut::<State>();

    if let Some(request) = state.measurement_request_to_publish() {
      shell.publish((self.on_measure_request)(request));
    }
  }
}

impl<'a, Message, Renderer> From<CodeView<'a, Message>> for Element<'a, Message, Theme, Renderer>
where
  Message: 'a,
  Renderer: 'a + RendererTrait + TextRendererTrait,
{
  fn from(code_view: CodeView<'a, Message>) -> Self {
    Self::new(code_view)
  }
}
