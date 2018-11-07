use mksvg::{SvgWrite,page,Card,Args,SvgArg};
use lazy_conf::{config,Loader,LzList,Getable};

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
        svg.rect(w*0.1,h*0.1,w*0.8,h*0.8,Args::new().stroke_width(w*0.05).stroke("black").fill("#bbbbbb"));
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

fn main()->Result<(),lazy_conf::LzErr>{
    let mut cf = config("-c",&["conf.lz"])?;
        
    //end in underscore_ shows still option
    let cardloc_ = cf.grab().cf("config.card_loc").help("Card Location").s_local();


    let linkpath_ = cf.grab().cf("config.link-path")
                        .help("Link Path -- the svgs relative path to img folder").s_req("Link Path");

    let fout = cf.grab().cf("config.out-front")
                    .help("Front Output -- base path for card fronts")
                    .s_local().unwrap_or(PathBuf::from("out/f"));
    let bout = cf.grab().cf("config.out-back")
                    .help("Back Output -- base path for card backs")
                    .s_local().unwrap_or(PathBuf::from("out/b"));

    let pdf_out_ = cf.grab().cf("config.out-pdf")
                    .help("final pdf result")
                    .s_local();

    if cf.help("Irrigate Card Maker"){
        return Ok(());
    }

    let linkpath = linkpath_.unwrap();//safe past help
    
    let cardlz = LzList::load(&cardloc_.unwrap())?;
    
    let mut fronts = Vec::new();
    let mut backs = Vec::new();

    for c_item in cardlz.items.iter(){
        let shapes = c_item.get("shapes").unwrap();//TODO ? somehow
        let count = c_item.get("count").unwrap().parse().unwrap();
        let cols = c_item.get("colors").unwrap();
        let back = c_item.get("back").unwrap();
        let cols = cf.grab().cf(&format!("colors.{}",cols)).s().unwrap();
        for i in 0..count{
            println!("Card:{}",i);
            for sh in shapes.split(',').map(|s|s.trim()){
                for col in cols.split(',').map(|s|s.trim()){
                    fronts.push(CFront{
                        shape:sh.to_string(),
                        col:col.to_string(),
                        cost:3, 
                        linkpath:linkpath.as_ref(),
                    });
                    backs.push(CBack{
                        tx:back.to_string(),
                    });
                }
            }
        }

    }

    //todo fix cols rows
    let fpages = page::pages_a4(fout,5,7,&fronts);

    let backs=  page::page_flip(&backs,5);

    let bpages = page::pages_a4(bout,5,7,&backs);

    let allpages = page::interlace(fpages,bpages);

    if let Some(p) = pdf_out_ {
        page::unite_as_pdf(allpages,p);
    }

    Ok(())
    
    //mksvg::page::page_a4(std::io::stdout(),5,7,&fronts);

}
