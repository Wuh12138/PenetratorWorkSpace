
fn test(){

    let mut vec_1=vec![];
    {
        let x=5;
        vec_1.push(x);
    }
    vec_1.pop();

    //let mut vec_2=vec![];
    let t;
    {
        let mut x=5;
        t=Test::new(&mut x);
    }


}

struct Test<T>{
    x:T,
}

impl<T> Test<T> {
    fn new(x:T)->Self{
        Self{
            x:x,
        }
        
    }

    fn print(&self){
        print!("fdsf");
    }
}
