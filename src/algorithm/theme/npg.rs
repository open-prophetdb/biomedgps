use once_cell::sync::Lazy;

use plotly::{
    common::{ColorBar, ColorScale, ColorScaleElement, Font, Label, Title},
    layout::{Axis, ColorAxis, HoverMode, LayoutColorScale, LayoutTemplate, Template},
};

pub static PLOTLY_NATURE: Lazy<Template> = Lazy::new(|| {
    let layout_template = LayoutTemplate::new()
        .color_axis(ColorAxis::new().color_bar(ColorBar::new().outline_width(0)))
        .color_scale(
            LayoutColorScale::new()
                .sequential(ColorScale::Vector(vec![
                    ColorScaleElement(0., "#0d0887".to_string()),
                    ColorScaleElement(0.1111111111111111, "#46039f".to_string()),
                    ColorScaleElement(0.2222222222222222, "#7201a8".to_string()),
                    ColorScaleElement(0.3333333333333333, "#9c179e".to_string()),
                    ColorScaleElement(0.4444444444444444, "#bd3786".to_string()),
                    ColorScaleElement(0.5555555555555556, "#d8576b".to_string()),
                    ColorScaleElement(0.6666666666666666, "#ed7953".to_string()),
                    ColorScaleElement(0.7777777777777778, "#fb9f3a".to_string()),
                    ColorScaleElement(0.8888888888888888, "#fdca26".to_string()),
                    ColorScaleElement(1., "#f0f921".to_string()),
                ]))
                .sequential_minus(ColorScale::Vector(vec![
                    ColorScaleElement(0., "#0d0887".to_string()),
                    ColorScaleElement(0.1111111111111111, "#46039f".to_string()),
                    ColorScaleElement(0.2222222222222222, "#7201a8".to_string()),
                    ColorScaleElement(0.3333333333333333, "#9c179e".to_string()),
                    ColorScaleElement(0.4444444444444444, "#bd3786".to_string()),
                    ColorScaleElement(0.5555555555555556, "#d8576b".to_string()),
                    ColorScaleElement(0.6666666666666666, "#ed7953".to_string()),
                    ColorScaleElement(0.7777777777777778, "#fb9f3a".to_string()),
                    ColorScaleElement(0.8888888888888888, "#fdca26".to_string()),
                    ColorScaleElement(1., "#f0f921".to_string()),
                ]))
                .diverging(ColorScale::Vector(vec![
                    ColorScaleElement(0., "#8e0152".to_string()),
                    ColorScaleElement(0.1, "#c51b7d".to_string()),
                    ColorScaleElement(0.2, "#de77ae".to_string()),
                    ColorScaleElement(0.3, "#f1b6da".to_string()),
                    ColorScaleElement(0.4, "#fde0ef".to_string()),
                    ColorScaleElement(0.5, "#f7f7f7".to_string()),
                    ColorScaleElement(0.6, "#e6f5d0".to_string()),
                    ColorScaleElement(0.7, "#b8e186".to_string()),
                    ColorScaleElement(0.8, "#7fbc41".to_string()),
                    ColorScaleElement(0.9, "#4d9221".to_string()),
                    ColorScaleElement(1., "#276419".to_string()),
                ])),
        )
        .colorway(vec![
            "#636efa", "#EF553B", "#00cc96", "#ab63fa", "#FFA15A", "#19d3f3", "#FF6692", "#B6E880",
            "#FF97FF", "#FECB52",
        ])
        .font(Font::new().color("#2a3f5f"))
        .hover_label(Label::new().align("left"))
        .hover_mode(HoverMode::Closest)
        .paper_background_color("#ffffff")
        .plot_background_color("#ffffff")
        .title(Title::new().x(0.05).font(Font::new().size(18)))
        .x_axis(
            Axis::new()
                .auto_margin(true)
                .grid_color("#EBF0F8")
                .line_color("black")
                .line_width(2)
                // missing title.standoff = 15
                // .zero_line_color("#EBF0F8")
                // .zero_line_width(1)
                .zero_line(false)
                .show_grid(false)
                .show_line(true)
                .tick_font(Font::new().size(18))
                .title(Title::new().font(Font::new().size(18))),
        )
        .y_axis(
            Axis::new()
                .auto_margin(true)
                .grid_color("#EBF0F8")
                .line_color("black")
                .line_width(2)
                // missing title.standoff = 15
                .zero_line(false)
                .show_grid(false)
                .show_line(true)
                .tick_font(Font::new().size(18))
                .title(Title::new().font(Font::new().size(18))),
        )
        .legend(plotly::layout::Legend::new().font(Font::new().size(18)));
    Template::new().layout(layout_template)
});
