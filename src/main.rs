extern crate lazyf;
extern crate mksvg;

use mksvg::{SvgWrite,page,Card,Args,SvgArg};
use lazyf::{Cfg,SGetter,LzList};

//use std::io::stdout;
use std::path::{PathBuf,Path};


#[derive(Clone)]
struct CFront<'a>{
    shape:String,
    col:String,
    cost:i32,
    linkpath:&'a Path,
}

impl<'a> Card<f64> for CFront<'a>{
    fn front<S:SvgWrite>(&self, svg:&mut S, w:f64,h:f64){
        svg.rect(0.0,0.0,w,h,Args::new().stroke_width(5).stroke("black").fill(&self.col));
        let mut imgloc = PathBuf::from(self.linkpath);
        imgloc.push(&format!("{}.svg",self.shape));
        let imgloc = imgloc.to_str().unwrap();
        svg.img(imgloc,0.0,0.0,w,h);
        if self.cost != 0 {
            for i in 0..4 {
                svg.g_rotate((i*90) as f64,w/2.0,h/2.0);
                svg.ellipse(w*0.11,h*0.11,w*0.08,w*0.08,
                    Args::new().stroke_width(w*0.02)
                    .fill("gold").stroke("black"));

                svg.bg_text(&self.cost.to_string(),w*0.11,h*0.16,
                    h*0.11,w*0.015,"white",
                    Args::new().t_anc("middle").fill("black").font_weight("bold"));
                svg.g_end();
            }
        }
    }
}


#[derive(Clone)]
struct CBack{
    tx:String,
}


impl Card<f64> for CBack{
    fn front<S:SvgWrite>(&self,svg:&mut S,w:f64,h:f64){
        svg.rect(0.0,0.0,w,h,Args::new().stroke_width(w*0.05).stroke("black").fill("#999999"));
        svg.rect(w*0.1,h*0.1,w*0.8,h*0.8,Args::new().stroke_width(w*0.05).stroke("black").fill("#ff0000"));
        let (tsize,tdepth)  = match self.tx.len() {
            1|2 => (h *0.2,h*0.28),
            _=> (h *0.13,h*0.2),
        };
        for i in 0..4 {
            svg.g_rotate((i*90) as f64,w*0.5,h*0.5);
            svg.bg_text(&self.tx,w*0.5,tdepth,tsize,tsize*0.2,"white",Args::new().font_weight("bold").t_anc("middle"));
            svg.g_end();
        }
    }
}

fn main(){
    let cf = Cfg::load_first("-c",&["conf.lz"]);
        
    let cardloc =cf.localize(&cf.get_s_def(("-cards","config.cards"),"cards.lz"));

    let cardlz = LzList::load(cardloc).unwrap();


    let ops = cf.get_s_def(("-","config.options"),"tee");

    let linkpath = cf.localize(&cf.get_s_def(("-lpath","config.link-path"),""));
    let mut fronts = Vec::new();
    let mut backs = Vec::new();


    
    for c in cardlz.iter(){
        let tp = c.get_s_def("type","person");
        //print!("<!--{}-->\n",tp);
        let cols = match cf.lz_by_name(&tp){
            Some(lz)=>lz.get_s_def("colors","#000000"),
            _=>{continue;},
        };
        let cost = c.get_t_def("cost",0);
        for op in ops.split(",") {
            match c.get_t::<usize>(op){
                Some(n)=>{
                    for _ in 0 ..n {
                        for col in cols.split(","){
                            //print!("  <!--{}-->\n",op.to_string());
                            fronts.push(CFront{
                                shape:op.to_string(),
                                col:col.to_string(),
                                cost:cost, 
                                linkpath:&linkpath,
                            });
                            backs.push(CBack{
                                tx:c.get_s_def("back","NONE"),
                            });
                        }
                    }
                }
                _=>{},
            }
        }//op in ops
    }
    let fout = cf.localize(&cf.get_s_def(("-fout","config.out-front"),"out/f"));
    let fpages = page::pages_a4(fout,5,7,&fronts);

    let bout = cf.localize(&cf.get_s_def(("-bout","config.out-back"),"out/f"));

    let backs=  page::page_flip(&backs,5);

    let bpages = page::pages_a4(bout,5,7,&backs);

    let allpages = page::interlace(fpages,bpages);

    page::unite_as_pdf(allpages,cf.localize(&cf.get_s_def(("-pdf","config.out-pdf"),"out/all.pdf")));

    
    //mksvg::page::page_a4(std::io::stdout(),5,7,&fronts);

}
