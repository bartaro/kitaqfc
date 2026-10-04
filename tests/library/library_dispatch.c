#include "entity.h"
__location(0x0700) u8 proof[16];
#pragma bank 0
void visit(u8 id) { proof[3]=(u8)(proof[3]+id+1); }
void main(void) {
    u8 id;
    Entity* e;
    proof[3]=0;
    entity_init();
    id=entity_create(1,12,34);
    proof[0]=id;
    id=entity_create(2,-12,-34);
    proof[1]=id;
    entity_destroy(0);
    proof[2]=entity_count_active();
    entity_update_all(visit);
    proof[4]=proof[3];
    entity_draw_all(visit);
    proof[5]=proof[3];
    proof[6]=(u8)(entity_get(0)==0);
    id=entity_create(3,56,78);
    e=entity_get(id);
    proof[7]=id;
    proof[8]=e->sprite;
    proof[9]=(u8)sizeof(Entity);
    proof[10]=0xA5;
    while (1) { }
}
