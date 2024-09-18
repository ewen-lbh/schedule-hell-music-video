use std::path::PathBuf;

use anyhow::Result;
use itertools::Itertools;
use shapemaker::*;

struct State {
    bass_pattern_at: Region,
    kick_color: Color,
}

impl Default for State {
    fn default() -> Self {
        Self {
            bass_pattern_at: Region::from_topleft(Point(1, 1), (2, 2)).unwrap(),
            kick_color: Color::White,
        }
    }
}

pub fn main() -> Result<()> {
    let mut canvas = Canvas::new(vec![]);

    canvas.set_grid_size(16, 9);
    canvas.colormap = ColorMapping {
        black: "#000000".into(),
        white: "#ffffff".into(),
        red: "#cf0a2b".into(),
        green: "#22e753".into(),
        blue: "#2734e6".into(),
        yellow: "#f8e21e".into(),
        orange: "#f05811".into(),
        purple: "#6a24ec".into(),
        brown: "#a05634".into(),
        pink: "#e92e76".into(),
        gray: "#81a0a8".into(),
        cyan: "#4fecec".into(),
    };

    let mut video = Video::<State>::new(canvas);
    let mut args = pico_args::Arguments::from_env();

    video.duration_override = args
        .value_from_str("--duration")
        .ok()
        .map(|seconds: usize| seconds * 1000);

    if video.duration_override == Some(0) {
        video.duration_override = None;
    }

    video.start_rendering_at = args
        .value_from_str("--start")
        .ok()
        .map(|seconds: usize| seconds * 1000)
        .unwrap_or_default();

    video.fps = 30;
    video.audiofile = PathBuf::from("../schedule-hell.flac");
    video = video
        .init(&|canvas, _| {
            canvas.set_background(Color::Black);

            let mut kicks = Layer::new("anchor kick");

            let circle_at = |x: usize, y: usize| Object::SmallCircle(Point(x, y));

            let (end_x, end_y) = {
                let Point(x, y) = canvas.world_region.end;
                (x - 2, y - 2)
            };
            kicks.set_object("top left", circle_at(1, 1).into());
            kicks.set_object("top right", circle_at(end_x, 1).into());
            kicks.set_object("bottom left", circle_at(1, end_y).into());
            kicks.set_object("bottom right", circle_at(end_x, end_y).into());
            canvas.add_or_replace_layer(kicks);

            let mut ch = Layer::new("ch");
            ch.set_object("0", Object::Dot(Point(0, 0)).into());
            canvas.add_or_replace_layer(ch);

            Ok(())
        })
        .sync_audio_with("../schedule-hell.midi")
        .on_note("anchor kick", &|canvas, ctx| {
            canvas
                .layer("anchor kick")
                .paint_all_objects(Fill::Translucent(ctx.extra.kick_color, 1.0));

            ctx.animate_layer("anchor kick", 200, &|t, layer, _| {
                layer
                    .objects
                    .values_mut()
                    .for_each(|ColoredObject { ref mut fill, .. }| {
                        *fill = fill.opacify(1.0 - t);
                    });
                Ok(())
            });

            Ok(())
        })
        .on_note("bass", &|canvas, ctx| {
            let mut new_layer = canvas.random_layer_within("bass", &ctx.extra.bass_pattern_at);
            new_layer.paint_all_objects(Fill::Solid(Color::White));
            canvas.add_or_replace_layer(new_layer);
            Ok(())
        })
        .on_note("powerful clap hit, clap, perclap", &|canvas, ctx| {
            let mut new_layer =
                canvas.random_layer_within("claps", &ctx.extra.bass_pattern_at.translated(2, 0));
            new_layer.paint_all_objects(Fill::Solid(Color::Red));
            canvas.add_or_replace_layer(new_layer);
            Ok(())
        })
        .on_note(
            "rimshot, glitchy percs, hitting percs, glitchy percs",
            &|canvas, ctx| {
                let mut new_layer = canvas
                    .random_layer_within("percs", &ctx.extra.bass_pattern_at.translated(2, 0));
                new_layer.paint_all_objects(Fill::Translucent(Color::Red, 0.5));
                canvas.add_or_replace_layer(new_layer);
                Ok(())
            },
        )
        .on_note("qanda", &|canvas, ctx| {
            let mut new_layer = canvas.random_linelikes_within(
                "qanda",
                &ctx.extra.bass_pattern_at.translated(-1, -1).enlarged(1, 1),
            );
            new_layer.paint_all_objects(Fill::Solid(Color::Orange));
            new_layer.object_sizes.default_line_width = canvas.object_sizes.default_line_width
                * 4.0
                * ctx.stem("qanda").velocity_relative();
            canvas.add_or_replace_layer(new_layer);
            Ok(())
        })
        .on_note("brokenup", &|canvas, ctx| {
            let mut new_layer = canvas
                .random_linelikes_within("brokenup", &ctx.extra.bass_pattern_at.translated(0, -2));
            new_layer.paint_all_objects(Fill::Solid(Color::Yellow));
            new_layer.object_sizes.default_line_width = canvas.object_sizes.default_line_width
                * 4.0
                * ctx.stem("brokenup").velocity_relative();
            canvas.add_or_replace_layer(new_layer);
            Ok(())
        })
        .on_note("goup", &|canvas, ctx| {
            let mut new_layer =
                canvas.random_linelikes_within("goup", &ctx.extra.bass_pattern_at.translated(0, 2));
            new_layer.paint_all_objects(Fill::Solid(Color::Green));
            new_layer.object_sizes.default_line_width =
                canvas.object_sizes.default_line_width * 4.0 * ctx.stem("goup").velocity_relative();
            canvas.add_or_replace_layer(new_layer);
            Ok(())
        })
        .on_note("ch", &|canvas, ctx| {
            let world = canvas.world_region.clone();

            // keep only the last 2 dots
            let dots_to_keep = canvas
                .layer("ch")
                .objects
                .iter()
                .sorted_by_key(|(name, _)| name.parse::<usize>().unwrap())
                .rev()
                .take(2)
                .map(|(name, _)| (name.clone()))
                .collect::<Vec<_>>();

            let layer = canvas.layer("ch");
            layer.object_sizes.empty_shape_stroke_width = 2.0;
            layer.objects.retain(|name, _| dots_to_keep.contains(name));

            let object_name = format!("{}", ctx.ms);
            layer.set_object(
                &object_name,
                Object::Dot(world.resized(-1, -1).random_coordinates_within().into())
                    .color(Fill::Solid(Color::Cyan)),
            );

            canvas.put_layer_on_top("ch");
            canvas.layer("ch").flush();
            Ok(())
        })
        .when_remaining(10, &|canvas, _| {
            let world = canvas.world_region;
            canvas.root().set_object(
                "credits text",
                Object::Text(world.start.translated(2, 2), "by ewen-lbh".into(), 12.0)
                    .color(Fill::Solid(Color::White)),
            );
            Ok(())
        })
        .command("remove", &|argumentsline, canvas, _| {
            let args = argumentsline.splitn(3, ' ').collect::<Vec<_>>();
            canvas.remove_object(args[0]);
            Ok(())
        });

    if args.contains("--preview") {
        video.preview_on(8888)?;
    } else {
        video.render_to("../schedule-hell.mp4".into(), num_cpus::get(), false)?;
    }

    Ok(())
}
