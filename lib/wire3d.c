// Copyright (c) 2026 DAISUKE OBA. SPDX-License-Identifier: MIT
// Independently implemented integer projection, pixel rasterization and
// double-buffered NES CHR upload. No commercial game code or data is included.
#include "wire3d.h"
#include "wire3d_tables.h"
__location(0x6800) u8 w3dfc_pixels[1536];
__location(0x6E00) u8 w3dfc_dirty[192];
__location(0x6F00) u8 w3dfc_bank0_dirty[192];
__location(0x7000) u8 w3dfc_bank1_dirty[192];
// Keep assembly scratch below the compiler local/call/temp windows ($10..$FF).
__location(0x0000) u16 w3dfc_upload_ptr;
__location(0x0002) u8 w3dfc_lx;
__location(0x0003) u8 w3dfc_ly;
__location(0x0004) u8 w3dfc_major;
__location(0x0005) u8 w3dfc_minor;
__location(0x0006) u8 w3dfc_error;
__location(0x0007) u8 w3dfc_remaining;
__location(0x0008) u8 w3dfc_sx;
__location(0x0009) u8 w3dfc_sy;
__location(0x000A) u8 w3dfc_y_major;
__location(0x2002) u8 w3dfc_status;
u8 w3dfc_front;
u8 w3dfc_tile_count;
u8 w3dfc_batch_count;
u8 w3dfc_src_lo[16];
u8 w3dfc_src_hi[16];
u8 w3dfc_dst_lo[16];
u8 w3dfc_dst_hi[16];
u8 wire3d_transfer_frames;
u8 wire3d_uploaded_tiles;
s16 w3dfc_projected_x[24];
s16 w3dfc_projected_y[24];
u8 w3dfc_valid[24];
__prg_rom const u8 w3dfc_bits[8]={128,64,32,16,8,4,2,1};
__prg_rom const u8 w3dfc_palette[16]={
    0x0F,0x2C,0x2C,0x2C,0x0F,0x2C,0x2C,0x2C,
    0x0F,0x2C,0x2C,0x2C,0x0F,0x2C,0x2C,0x2C
};

void Wire3DFC_Init(void) {
    u16 address;u8 x,y,tile;
    __ppu_mask_set(0);__ppu_ctrl_set(0);
    // Both pattern tables have a zero high bitplane; all subsequent transfers
    // update only low bitplanes in the currently hidden table.
    for(address=0;address<8192;address+=128)__vram_fill(address,0,128);
    for(address=0x2000;address<0x2400;address+=128)__vram_fill(address,255,128);
    __vram_fill(0x23C0,0,64);__palette_bg_load(w3dfc_palette);
    tile=0;
    for(y=0;y<WIRE3D_FC_HEIGHT/8;y++) {
        for(x=0;x<WIRE3D_FC_WIDTH/8;x++) {
            address=(u16)(0x2000+(u16)(y+(30-WIRE3D_FC_HEIGHT/8)/2)*32+x+(32-WIRE3D_FC_WIDTH/8)/2);
            __ppu_addr(address);__ppu_data(tile);tile++;
        }
    }
    __memset(w3dfc_pixels,0,1536);
    __memset(w3dfc_dirty,0,192);
    __memset(w3dfc_bank0_dirty,0,192);
    __memset(w3dfc_bank1_dirty,0,192);
    w3dfc_front=0;w3dfc_batch_count=0;
    wire3d_transfer_frames=0;wire3d_uploaded_tiles=0;
    __scroll_set(0,0);__ppu_ctrl_set(0);__ppu_mask_set(0x0A);
}
void Wire3DFC_BeginFrame(void) {
    w3dfc_tile_count=WIRE3D_FC_TILES;
    // Clear exactly eight bytes per previously touched tile, using the same
    // zero-page pointer as upload. No nested C indexing runs in the inner loop.
    __asm {
        PHA
        TXA
        PHA
        TYA
        PHA
        LDX #0
w3dfc_clear_next:
        LDA w3dfc_dirty,X
        BEQ w3dfc_clear_skip
        TXA
        ASL A
        ASL A
        ASL A
        STA w3dfc_upload_ptr
        TXA
        LSR A
        LSR A
        LSR A
        LSR A
        LSR A
        CLC
        ADC #104
        STA w3dfc_upload_ptr+1
        LDA #0
        STA w3dfc_dirty,X
        LDY #7
w3dfc_clear_byte:
        STA (w3dfc_upload_ptr),Y
        DEY
        BPL w3dfc_clear_byte
w3dfc_clear_skip:
        INX
        CPX w3dfc_tile_count
        BNE w3dfc_clear_next
        PLA
        TAY
        PLA
        TAX
        PLA
    }
}
// In-viewport lines need no per-pixel clipping or word arithmetic. A midpoint
// accumulator yields the same inclusive Bresenham pixels as the clipped path.
// Row tables replace tile multiplication; all scratch is non-reentrant.
void w3dfc_line_inside(void) {
    __asm {
        PHA
        TXA
        PHA
        TYA
        PHA
w3dfc_line_plot:
        LDY w3dfc_ly
        LDA w3dfc_lx
        AND #248
        CLC
        ADC w3dfc_row_lo,Y
        STA w3dfc_upload_ptr
        LDA w3dfc_row_hi,Y
        ADC #0
        STA w3dfc_upload_ptr+1
        LDA w3dfc_lx
        LSR A
        LSR A
        LSR A
        CLC
        ADC w3dfc_tile_row,Y
        TAX
        LDA #1
        STA w3dfc_dirty,X
        LDA w3dfc_lx
        AND #7
        TAX
        LDA w3dfc_bits,X
        LDY #0
        ORA (w3dfc_upload_ptr),Y
        STA (w3dfc_upload_ptr),Y
        DEC w3dfc_remaining
        BEQ w3dfc_line_done
        LDA w3dfc_y_major
        BNE w3dfc_line_step_y
        LDA w3dfc_lx
        CLC
        ADC w3dfc_sx
        STA w3dfc_lx
        LDA w3dfc_error
        SEC
        SBC w3dfc_minor
        BCS w3dfc_line_store_error
        CLC
        ADC w3dfc_major
        STA w3dfc_error
        LDA w3dfc_ly
        CLC
        ADC w3dfc_sy
        STA w3dfc_ly
        JMP w3dfc_line_plot
w3dfc_line_step_y:
        LDA w3dfc_ly
        CLC
        ADC w3dfc_sy
        STA w3dfc_ly
        LDA w3dfc_error
        SEC
        SBC w3dfc_minor
        BCS w3dfc_line_store_error
        CLC
        ADC w3dfc_major
        STA w3dfc_error
        LDA w3dfc_lx
        CLC
        ADC w3dfc_sx
        STA w3dfc_lx
        JMP w3dfc_line_plot
w3dfc_line_store_error:
        STA w3dfc_error
        JMP w3dfc_line_plot
w3dfc_line_done:
        PLA
        TAY
        PLA
        TAX
        PLA
    }
}
void Wire3DFC_DrawLine2D(s16 ax,s16 ay,s16 bx,s16 by) {
    s16 dx,dy,sx,sy,error,twice;u16 address;u8 tile;
    if(ax < -512 || ax>511 || ay < -512 || ay>511 || bx < -512 || bx>511 || by < -512 || by>511)return;
    if((ax<0 && bx<0)||(ay<0 && by<0)||(ax>=WIRE3D_FC_WIDTH && bx>=WIRE3D_FC_WIDTH)||(ay>=WIRE3D_FC_HEIGHT && by>=WIRE3D_FC_HEIGHT))return;
    dx=bx-ax;if(dx<0)dx=0-dx;
    dy=by-ay;if(dy<0)dy=0-dy;
    if(ax>=0 && ay>=0 && bx>=0 && by>=0 && ax<WIRE3D_FC_WIDTH && bx<WIRE3D_FC_WIDTH && ay<WIRE3D_FC_HEIGHT && by<WIRE3D_FC_HEIGHT) {
        w3dfc_lx=(u8)ax;w3dfc_ly=(u8)ay;
        w3dfc_sx=ax<bx?1:255;w3dfc_sy=ay<by?1:255;
        if(dx>=dy){w3dfc_major=(u8)dx;w3dfc_minor=(u8)dy;w3dfc_y_major=0;}
        else{w3dfc_major=(u8)dy;w3dfc_minor=(u8)dx;w3dfc_y_major=1;}
        w3dfc_error=(u8)(w3dfc_major>>1);w3dfc_remaining=(u8)(w3dfc_major+1);
        w3dfc_line_inside();return;
    }
    sx=ax<bx?1:-1;sy=ay<by?1:-1;error=dx-dy;
    while(1) {
        if(ax>=0 && ax<WIRE3D_FC_WIDTH && ay>=0 && ay<WIRE3D_FC_HEIGHT) {
            tile=(u8)(((u8)ay>>3)*(WIRE3D_FC_WIDTH/8)+((u8)ax>>3));
            address=(u16)tile*8+((u8)ay&7);
            w3dfc_pixels[address]=(u8)(w3dfc_pixels[address]|w3dfc_bits[(u8)ax&7]);
            w3dfc_dirty[tile]=1;
        }
        if(ax==bx && ay==by)return;
        twice=error*2;
        if(twice > (0-dy)){error-=dy;ax+=sx;}
        if(twice < dx){error+=dx;ay+=sy;}
    }
}
// A fixed Q6 reduction truncates toward zero, including negative products.
// The bias permits an arithmetic shift instead of a general word division.
s16 w3dfc_q6(s16 value) {
    if(value<0)value+=63;
    return value>>6;
}
void Wire3DFC_RotatePoint(s16* x,s16* y,s16* z,u8 rx,u8 ry,u8 rz) {
    s16 a,b,s,c;
    if(x==0 || y==0 || z==0)return;
    ry=(u8)(ry&31);rx=(u8)(rx&31);rz=(u8)(rz&31);
    if(ry!=0){a=*x;b=*z;s=w3dfc_sin[ry];c=w3dfc_sin[(u8)((ry+8)&31)];*x=w3dfc_q6(a*c+b*s);*z=w3dfc_q6(b*c-a*s);}
    if(rx!=0){a=*y;b=*z;s=w3dfc_sin[rx];c=w3dfc_sin[(u8)((rx+8)&31)];*y=w3dfc_q6(a*c-b*s);*z=w3dfc_q6(a*s+b*c);}
    if(rz!=0){a=*x;b=*y;s=w3dfc_sin[rz];c=w3dfc_sin[(u8)((rz+8)&31)];*x=w3dfc_q6(a*c-b*s);*y=w3dfc_q6(a*s+b*c);}
}
u8 Wire3DFC_ProjectPoint(s16 x,s16 y,s16 z,s16* sx,s16* sy) {
    u8 scale;u16 magnitude;
    if(sx==0 || sy==0 || x < -127 || x>127 || y < -127 || y>127 || z<32 || z>255)return 0;
    // Focal length follows half the selected width. Use unsigned magnitude
    // products and shifts so negative coordinates mirror positive ones exactly.
    scale=w3dfc_recip[(u8)z];
    if(x<0){magnitude=__mul16x8((u16)(0-x),scale);*sx=WIRE3D_FC_WIDTH/2-(s16)(magnitude>>7);}
    else{magnitude=__mul16x8((u16)x,scale);*sx=WIRE3D_FC_WIDTH/2+(s16)(magnitude>>7);}
    if(y<0){magnitude=__mul16x8((u16)(0-y),scale);*sy=WIRE3D_FC_HEIGHT/2+(s16)(magnitude>>7);}
    else{magnitude=__mul16x8((u16)y,scale);*sy=WIRE3D_FC_HEIGHT/2-(s16)(magnitude>>7);}
    return 1;
}
void Wire3DFC_DrawLine3D(s16 ax,s16 ay,s16 az,s16 bx,s16 by,s16 bz) {
    s16 x0,y0,x1,y1;
    if(Wire3DFC_ProjectPoint(ax,ay,az,&x0,&y0)==0)return;
    if(Wire3DFC_ProjectPoint(bx,by,bz,&x1,&y1)==0)return;
    Wire3DFC_DrawLine2D(x0,y0,x1,y1);
}
void Wire3DFC_DrawModel(const Wire3DFC_Vec3* vertices,u8 vertex_count,const Wire3DFC_Edge* edges,u8 edge_count,s16 x,s16 y,s16 z,u8 rx,u8 ry,u8 rz) {
    u8 i,a,b;s16 px,py,pz;
    if(vertices==0 || edges==0)return;
    if(vertex_count>24)vertex_count=24;
    for(i=0;i<vertex_count;i++) {
        px=vertices[i].x;py=vertices[i].y;pz=vertices[i].z;
        Wire3DFC_RotatePoint(&px,&py,&pz,rx,ry,rz);
        w3dfc_valid[i]=Wire3DFC_ProjectPoint(px+x,py+y,pz+z,&w3dfc_projected_x[i],&w3dfc_projected_y[i]);
    }
    for(i=0;i<edge_count;i++) {
        a=edges[i].a;b=edges[i].b;
        if(a<vertex_count && b<vertex_count && w3dfc_valid[a]!=0 && w3dfc_valid[b]!=0)
            Wire3DFC_DrawLine2D(w3dfc_projected_x[a],w3dfc_projected_y[a],w3dfc_projected_x[b],w3dfc_projected_y[b]);
    }
}

// The complete upload loop is bounded to sixteen tiles (128 PPUDATA bytes).
// Prepare addresses before waiting so scanning dirty flags cannot overrun VBlank.
void w3dfc_upload_batch(void) {
    __asm {
        PHA
        TXA
        PHA
        TYA
        PHA
w3dfc_wait_visible:
        LDA $2002
        BMI w3dfc_wait_visible
w3dfc_wait_blank:
        LDA $2002
        BPL w3dfc_wait_blank
        LDX #0
w3dfc_upload_tile:
        LDA w3dfc_src_lo,X
        STA w3dfc_upload_ptr
        LDA w3dfc_src_hi,X
        STA w3dfc_upload_ptr+1
        LDA w3dfc_dst_hi,X
        STA $2006
        LDA w3dfc_dst_lo,X
        STA $2006
        LDY #0
        LDA (w3dfc_upload_ptr),Y
        STA $2007
        INY
        LDA (w3dfc_upload_ptr),Y
        STA $2007
        INY
        LDA (w3dfc_upload_ptr),Y
        STA $2007
        INY
        LDA (w3dfc_upload_ptr),Y
        STA $2007
        INY
        LDA (w3dfc_upload_ptr),Y
        STA $2007
        INY
        LDA (w3dfc_upload_ptr),Y
        STA $2007
        INY
        LDA (w3dfc_upload_ptr),Y
        STA $2007
        INY
        LDA (w3dfc_upload_ptr),Y
        STA $2007
        INX
        CPX w3dfc_batch_count
        BNE w3dfc_upload_tile
        // Restore nametable scroll after PPUADDR writes, retaining the front table.
        LDA w3dfc_front
        STA $2000
        LDA #0
        STA $2005
        STA $2005
        PLA
        TAY
        PLA
        TAX
        PLA
    }
    wire3d_transfer_frames++;
    w3dfc_batch_count=0;
}
void Wire3DFC_EndFrame(void) {
    u8 tile,back,old;u16 source,destination;
    back=(u8)(w3dfc_front^16);wire3d_transfer_frames=0;wire3d_uploaded_tiles=0;w3dfc_batch_count=0;
    for(tile=0;tile<WIRE3D_FC_TILES;tile++) {
        if(back==0)old=w3dfc_bank0_dirty[tile];else old=w3dfc_bank1_dirty[tile];
        if(old!=0 || w3dfc_dirty[tile]!=0) {
            source=(u16)(0x6800+(u16)tile*8);
            destination=(u16)((u16)back*256+(u16)tile*16);
            w3dfc_src_lo[w3dfc_batch_count]=(u8)source;w3dfc_src_hi[w3dfc_batch_count]=(u8)(source>>8);
            w3dfc_dst_lo[w3dfc_batch_count]=(u8)destination;w3dfc_dst_hi[w3dfc_batch_count]=(u8)(destination>>8);
            w3dfc_batch_count++;wire3d_uploaded_tiles++;
            if(back==0)w3dfc_bank0_dirty[tile]=w3dfc_dirty[tile];else w3dfc_bank1_dirty[tile]=w3dfc_dirty[tile];
            if(w3dfc_batch_count==16)w3dfc_upload_batch();
        }
    }
    if(w3dfc_batch_count!=0)w3dfc_upload_batch();
    // Switch pattern tables only at a fresh VBlank, after every transfer finishes.
    while((w3dfc_status&128)!=0){}
    while((w3dfc_status&128)==0){}
    w3dfc_front=back;__ppu_ctrl_set(back);__scroll_set(0,0);
    wire3d_transfer_frames++;
}
