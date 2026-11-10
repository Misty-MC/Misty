package online.mistylauncher.client.menu;

/**
 * The night landscape as data, for the loading screen (the textures of
 * the mod cannot be read while the game loads its resources). Written by the launcher:
 * {@code misty-rust/src/ui/scene.rs}, {@code exporter_paysage_du_mod}. Do not edit by hand.
 */
final class SceneData {
	/** Ridges (far to near): per column of 200, the filled rows as pairs of letters ('A' = row 0, end excluded). */
	static final String[] RIDGES = {
		"dx,dx,dx,dx,cx,cx,dx,cx,cx,cx,bx,bx,ax,ax,`x,`x,`x,`x,`x,`x,_x,_x,`x,`x,`x,ax,ax,ax,ax,bx,bx,bx,bx,cx,bx,ax,ax,ax,bx,ax,ax,ax,`x,_x,`x,ax,ax,`x,ax,ax,ax,`x,`x,`x,`x,_x,_x,_x,]x,]x,\\x,\\x,[x,[x,[x,Zx,Zx,Yx,Yx,Wx,Wx,Wx,Vx,Vx,Vx,Wx,Wx,Wx,Xx,Yx,Yx,Yx,Zx,Zx,Zx,[x,[x,[x,[x,[x,[x,[x,[x,\\x,]x,]x,]x,]x,]x,^x,]x,^x,^x,_x,`x,ax,`x,`x,ax,bx,bx,cx,cx,cx,bx,bx,`x,`x,`x,_x,_x,_x,_x,^x,^x,^x,^x,^x,^x,^x,^x,_x,_x,_x,_x,ax,`x,`x,ax,ax,bx,bx,bx,cx,cx,cx,cx,cx,bx,bx,bx,bx,bx,bx,cx,cx,cx,dx,dx,dx,ex,ex,ex,fx,fx,fx,gx,gx,fx,fx,fx,ex,dx,cx,cx,bx,bx,bx,ax,ax,`x,`x,_x,^x,_x,_x,^x,^x,^x,]x,]x,]x,^x,]x,]x,]x,\\x,\\x,\\x,[x",
		"`x,ax,ax,bx,cx,cx,cx,dx,ex,ex,ex,fx,fx,fx,gx,gx,gx,hx,ix,jx,kx,kx,kx,kx,kx,jx,jx,ix,ix,hx,hx,hx,hx,hx,hx,hx,hx,hx,hx,ix,ix,jx,jx,jx,jx,ix,ix,ix,ix,ix,ix,jx,jx,ix,ix,jx,kx,kx,kx,jx,ix,gx,gx,fx,fx,fx,ex,ex,dx,dx,dx,bx,bx,ax,ax,ax,ax,`x,`x,`x,_x,_x,_x,_x,_x,_x,`x,bx,cx,dx,ex,fx,ex,fx,fx,fx,fx,fx,fx,fx,ex,ex,ex,ex,fx,gx,gx,gx,hx,hx,hx,hx,hx,hx,gx,gx,gx,gx,gx,gx,gx,hx,ix,ix,jx,kx,kx,lx,lx,lx,mx,mx,mx,lx,lx,lx,kx,kx,kx,kx,kx,kx,kx,jx,jx,ix,gx,gx,fx,ex,dx,cx,bx,bx,ax,ax,ax,bx,ax,ax,bx,bx,bx,cx,cx,cx,dx,dx,cx,cx,cx,cx,dx,dx,dx,ex,ex,ex,fx,fx,fx,fx,ex,dx,dx,cx,cx,ax,bx,cx,cx,bx,bx,dx,dx,ex,fx,ex,fx,fx",
		"mx,lx,kx,lx,ox,ox,ox,ox,ox,px,px,ox,nx,oqrx,rx,rx,rx,rx,rx,qx,rx,qx,qx,ox,mx,kx,jx,kx,mx,mx,mx,kx,jx,kx,kx,jx,jx,jx,kx,kx,kx,ix,hx,ilmx,klmx,mx,klmx,ix,hx,ix,kx,hjkx,gx,hx,ix,ix,jx,jx,kx,jx,ix,ix,jx,jx,ix,ix,gx,fx,gjkx,ijlx,mx,nx,nx,nx,mx,nx,px,ox,ox,ox,mx,lx,mx,ox,px,px,px,px,ox,lmox,jmox,ix,jx,lmnx,nx,nx,ox,ox,ox,ox,mopx,lx,mx,ox,nx,mx,lx,kx,lx,nx,nx,nx,nx,mx,jlmx,ix,jx,kx,jx,jx,ix,ix,ix,jx,jx,kx,kx,kx,jx,ix,jx,lx,kx,jx,kx,nx,ox,mx,lx,mx,ox,px,ox,lmnx,jmnx,ix,jx,lx,jx,ix,jx,lx,nx,nx,nx,mx,lx,mx,nx,nx,ox,ox,ox,ox,ox,ox,px,nopx,lx,kx,lx,nx,mx,lx,lx,kx,hjkx,gx,hx,kx,kx,lx,ijkx,gjkx,fx,gx,ix,jx,jx,jx,ix,hx,ix,kx,lx,kx,kx,lx,kx,kx",
		"rx,qx,qx,ox,mx,lx,mx,orsx,qrsx,rx,rx,rx,rx,sx,sx,sx,tx,sx,qx,px,qx,sx,ux,vx,vx,vx,ux,ux,ux,ux,tx,rstx,px,nx,mx,nx,px,rx,sx,sx,sx,sx,rx,rx,qx,orsx,mrsx,lx,mrsx,orsx,qrsx,rx,sx,sx,rx,qx,px,qx,qx,qx,px,ox,px,rx,tx,tx,ux,ux,sx,rx,sx,ux,vx,tx,tx,tx,sx,tx,tx,qrtx,ortx,mrsx,lx,mx,ox,qx,qx,qx,qx,ox,mx,lx,mx,ox,qx,qx,qx,qx,qx,qx,rx,opqx,mx,kx,jx,kx,mx,opqx,qx,rx,sx,sx,tx,ux,ux,tx,rx,px,ox,puvx,ruvx,sx,rx,sx,tx,tx,sx,sx,tx,ux,ux,ux,ux,sx,qx,px,qx,sx,tx,tx,tx,tx,qstx,px,qx,sx,qx,px,px,ox,mx,kx,jx,kx,mx,nx,mx,nx,qx,qx,rx,rx,rx,px,ox,px,px,nx,mx,nrsx,prsx,ux,sx,qx,px,qx,sx,sx,rx,sx,vx,vx,wx,wx,vx,vx,vx,ux,ux,tx,sx,qx,px,qx,rx,rx,qx,qx,qx,qx"
	};
	/** Stars (800 × 220 grid): "x.y" of each lit pixel. */
	static final String STARS = "248.0 300.0 348.4 796.4 797.4 796.5 797.5 160.8 788.8 128.16 320.16 476.16 568.16 569.16 568.17 569.17 192.20 592.20 740.20 741.20 740.21 741.21 196.24 428.24 172.28 580.28 581.28 604.28 580.29 581.29 424.36 425.36 424.37 425.37 252.40 380.40 381.40 416.40 480.40 504.40 532.40 380.41 381.41 700.44 36.48 37.48 184.48 36.49 37.49 116.52 508.52 509.52 632.52 508.53 509.53 752.56 228.60 540.64 628.64 632.64 392.68 393.68 668.68 392.69 393.69 264.72 12.76 628.80 84.84 476.84 488.84 489.84 488.85 489.85 104.88 224.88 225.88 232.88 224.89 225.89 324.92 332.92 333.92 560.92 604.92 605.92 700.92 756.92 332.93 333.93 604.93 605.93 16.96 328.96 532.96 564.96 565.96 796.96 564.97 565.97 96.100";
	static final String TWINKLE = "752.4 480.8 188.12 189.12 304.12 692.12 188.13 189.13 100.16 124.20 676.20 712.20 768.20 769.20 768.21 769.21 332.28 784.28 608.36 228.48 784.48 464.52 88.56 89.56 148.56 588.56 88.57 89.57 616.60 148.72 149.72 652.72 148.73 149.73 272.76 392.76 212.80 164.88 532.88 80.92 264.96 376.96 404.96";
	/** Moon, 11 × 11: 0 empty, 1 lit, 2 crater. */
	static final String MOON = "0000010000000111111100011111211100111111111001122111210111211111110111111111001111112110011111111100011111110000000100000";

	private SceneData() {
	}
}
