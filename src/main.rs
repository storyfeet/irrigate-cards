use clap_conf::prelude::*;
use failure_derive::*;
use lazy_conf::{Getable, Loader, LzList};
use mksvg::{page, Args, Card, SvgArg, SvgWrite};

//use std::io::stdout;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
struct CFront<'a> {
    shape: String,
    col: String,
    pic: Option<String>,
    linkpath: &'a Path,
}

#[derive(Debug, Fail)]
#[fail(display = "e{}", 0)]
struct StErr(&'static str);

impl<'a> Card<f64> for CFront<'a> {
    fn front<S: SvgWrite>(&self, svg: &mut S, w: f64, h: f64) {
        svg.rect(
            0.0,
            0.0,
            w,
            h,
            Args::new().stroke_width(5).stroke("black").fill(&self.col),
        );
        if let Some(ref p) = self.pic {
            let mut imgloc = PathBuf::from(self.linkpath);
            imgloc.push(p);
            let imgloc = imgloc.to_str().unwrap();
            svg.img(imgloc, 1., 1., w - 2., h - 2.);
        }
        let mut imgloc = PathBuf::from(self.linkpath);
        imgloc.push(&self.shape);
        let imgloc = imgloc.to_str().unwrap();
        svg.img(imgloc, 0.0, 0.0, w, h);
    }
}

#[derive(Clone)]
struct CBack {
    tx: String,
}

impl Card<f64> for CBack {
    fn front<S: SvgWrite>(&self, svg: &mut S, w: f64, h: f64) {
        svg.rect(
            0.0,
            0.0,
            w,
            h,
            Args::new()
                .stroke_width(w * 0.05)
                .stroke("black")
                .fill("#999999"),
        );
        svg.rect(
            w * 0.1,
            h * 0.1,
            w * 0.8,
            h * 0.8,
            Args::new()
                .stroke_width(w * 0.05)
                .stroke("black")
                .fill("#bbbbbb"),
        );
        let tsize = match self.tx.len() {
            1 | 2 => h * 0.5,
            _ => h * 0.3,
        };

        svg.bg_text(
            &self.tx,
            w * 0.5,
            h * 0.5 + tsize * 0.3,
            tsize,
            tsize * 0.2,
            "white",
            Args::new().font_weight("bold").t_anc("middle"),
        );
    }
}

fn main() -> Result<(), failure::Error> {
    //    let mut cf = config("-c", &["conf.lz"])?;

    let clp = clap_app! (irrigate=>
        (about:"Makes cards for Pipeland Game")
        (version:crate_version!())
        (author:"Matthew Stoodley")
        (@arg config:-c +takes_value "Location of config file")
    )
    .get_matches();

    let cfg = with_toml_env(&clp, &["conf.toml"]);
    //end in underscore_ shows still option
    println!("{:?}", cfg);

    let cardloc = cfg
        .grab_local()
        .conf("config.card_loc")
        .done()
        .ok_or(StErr("Please Provide Card Loc 'config.card_loc'"))?;

    let linkpath = cfg
        .grab()
        .conf("config.link-path")
        .done()
        .ok_or(StErr("Please provide link path 'config.link-path'"))?;

    let fout = cfg //todo work out localise
        .grab_local()
        .conf("config.out-front")
        .def("out/b");

    let bout = cfg //todo work out localise
        .grab_local()
        .conf("config.out-back")
        .def("out/b");

    let pdf_out_ = cfg //todo work out localise
        .grab_local()
        .conf("config.out-pdf")
        .done();

    println!("cardloc = {:?}", cardloc);
    let cardlz = LzList::load(&cardloc)?;
    println!("Cardlz = {:?}", cardlz);

    let mut fronts = Vec::new();
    let mut backs = Vec::new();

    for c_item in cardlz.items.iter() {
        println!("c_item == {:?}", c_item);
        let shapes = c_item.get("shapes").unwrap(); //TODO ? somehow
        let count = c_item.get("count").unwrap().parse().unwrap();
        let col_mems = c_item.get("colors").unwrap();
        let cbacks = c_item.get("backs").unwrap();
        let cols = cfg.grab().conf(&format!("colors.{}", col_mems)).done();
        let pics = cfg.grab().conf(&format!("pics.{}", col_mems)).done();
        for bak in cbacks.split(',').map(|s| s.trim()) {
            for sh in shapes.split(',').map(|s| s.trim()) {
                let sh = cfg
                    .grab()
                    .conf(&format!("shapes.{}", sh))
                    .def(format!("{}.svg", sh));
                for _ in 0..count {
                    if let Some(ref cols) = cols {
                        for col in cols.split(',').map(|s| s.trim()) {
                            fronts.push(CFront {
                                shape: sh.clone(),
                                col: col.to_string(),
                                pic: None,
                                linkpath: linkpath.as_ref(),
                            });
                            backs.push(CBack {
                                tx: bak.to_string(),
                            });
                        }
                    }
                    if let Some(ref pics) = pics {
                        for pic in pics.split(',').map(|s| s.trim()) {
                            fronts.push(CFront {
                                shape: sh.clone(),
                                col: "none".to_string(),
                                pic: Some(pic.to_string()),
                                linkpath: linkpath.as_ref(),
                            });
                            backs.push(CBack {
                                tx: bak.to_string(),
                            });
                        }
                    }
                }
            }
        }
    }

    println!("fronts = {:?}", fronts);
    //todo fix cols rows
    let fpages = page::pages_a4(fout, 5, 7, &fronts)?;

    let backs = page::page_flip(&backs, 5);

    let bpages = page::pages_a4(bout, 5, 7, &backs)?;

    let allpages = page::interlace(fpages, bpages);

    if let Some(p) = pdf_out_ {
        page::unite_as_pdf(allpages, p);
    }

    Ok(())

    //mksvg::page::page_a4(std::io::stdout(),5,7,&fronts);
}
