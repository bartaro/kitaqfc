// Copyright (c) 2026 DAISUKE OBA. MIT. Adapter to BSD-3-Clause ymfm.
#include "ymfm_opn.h"
#include "ymfm_misc.h"
#include <algorithm>
#include <array>
#include <cmath>
#include <cstdint>
#include <new>

// Original synthesized percussion, not a dump of Yamaha's embedded sample ROM.
// The register interface and ADPCM decoder are unchanged. Hosts may supply an
// 8192-byte ROM they are entitled to use through fcseq_ym_rom.
static std::array<uint8_t,8192> percussion() {
    std::array<uint8_t,8192> out{};
    const unsigned starts[]={0,0x1c0,0x440,0x1b80,0x1d00,0x1f80,0x2000};
    const int shifts[]={-1,-1,-1,-1,2,5,7,9};
    uint32_t noise=0x62731;
    for(unsigned drum=0;drum<6;drum++) {
        int accum=0,index=0;
        double phase=0;
        unsigned n=(starts[drum+1]-starts[drum])*2;
        for(unsigned i=0;i<n;i++) {
            double t=i/18518.5;
            noise^=noise<<13;noise^=noise>>17;noise^=noise<<5;
            double rnd=(int32_t(noise)/2147483648.0);
            double freq=(drum==0?55+100*std::exp(-t*45):drum==4?140:850);
            phase+=6.283185307179586*freq/18518.5;
            double tone=std::sin(phase);
            double y=drum==0?tone*std::exp(-t*18):
                drum==1?(0.7*rnd+0.3*tone)*std::exp(-t*18):
                drum==2?rnd*std::exp(-t*7):
                drum==3?rnd*std::exp(-t*70):
                drum==4?tone*std::exp(-t*16):(tone+0.4*rnd)*std::exp(-t*120);
            int target=int(y*1500),step=int(std::floor(16*std::pow(1.1,index)));
            int code=0,best=100000,bvalue=0;
            for(int v=0;v<16;v++) {
                int delta=((2*(v&7)+1)*step)/8;
                int candidate=std::max(-2048,std::min(2047,accum+((v&8)?-delta:delta)));
                int err=std::abs(target-candidate);
                if(err<best){best=err;code=v;bvalue=candidate;}
            }
            accum=bvalue;index=std::max(0,std::min(48,index+shifts[code&7]));
            out[starts[drum]+i/2]|=uint8_t(code<<((i&1)?0:4));
        }
    }
    return out;
}

struct Host : ymfm::ymfm_interface {
    bool epsm,irq=false;
    uint64_t time=0,busy=0,frac=0,sample_frac=0;
    int64_t timers[2]={-1,-1};
    int32_t sample=0;
    std::array<uint8_t,8192> rom;
    ymfm::ymf288 opn;
    ymfm::ym2149 ssg;
    explicit Host(bool e):epsm(e),rom(e?percussion():std::array<uint8_t,8192>{}),opn(*this),ssg(*this) {
        if(epsm){opn.set_fidelity(ymfm::OPN_FIDELITY_MIN);opn.reset();}
        else ssg.reset();
    }
    void ymfm_set_timer(uint32_t n,int32_t duration) override {timers[n]=duration<0?-1:int64_t(time)+duration;}
    void ymfm_set_busy_end(uint32_t clocks) override {busy=time+clocks;}
    bool ymfm_is_busy() override {return time<busy;}
    void ymfm_update_irq(bool value) override {irq=value;}
    uint8_t ymfm_external_read(ymfm::access_class type,uint32_t address) override {
        return type==ymfm::ACCESS_ADPCM_A?rom[address&8191]:0;
    }
    void clock(uint32_t cycles) {
        // 5B YM2149 SEL is low: half the CPU clock. EPSM crystal is 8 MHz.
        const uint64_t clock2=epsm?16000000:1789773;
        frac+=uint64_t(cycles)*clock2;
        uint64_t clocks=frac/(2*1789773);frac%=2*1789773;
        const unsigned divisor=epsm?144:8;
        for(uint64_t i=0;i<clocks;i++) {
            ++time;
            for(unsigned n=0;n<2;n++) if(timers[n]>=0 && uint64_t(timers[n])<=time) {
                timers[n]=-1;m_engine->engine_timer_expired(n);
            }
            if(++sample_frac>=divisor) {
                sample_frac-=divisor;
                if(epsm) {
                    ymfm::ymf288::output_data output;opn.generate(&output);
                    sample=(output.data[0]+output.data[1])/2+output.data[2];
                } else {
                    ymfm::ym2149::output_data output;ssg.generate(&output);
                    sample=(output.data[0]+output.data[1]+output.data[2])/2;
                }
            }
        }
    }
};
extern "C" {
void *fcseq_ym_new(uint8_t epsm){return new(std::nothrow) Host(epsm!=0);}
void fcseq_ym_delete(void *p){delete static_cast<Host*>(p);}
void fcseq_ym_write(void *p,uint8_t port,uint8_t value){auto &h=*static_cast<Host*>(p);if(h.epsm)h.opn.write(port&3,value);else if(port&1)h.ssg.write_data(value);else h.ssg.write_address(value);}
uint8_t fcseq_ym_read(void *p,uint8_t port){auto &h=*static_cast<Host*>(p);return h.epsm?h.opn.read(port&3):h.ssg.read(port&1);}
void fcseq_ym_clock(void *p,uint32_t cycles){static_cast<Host*>(p)->clock(cycles);}
int32_t fcseq_ym_sample(void *p){return static_cast<Host*>(p)->sample;}
uint8_t fcseq_ym_irq(void *p){return static_cast<Host*>(p)->irq;}
void fcseq_ym_rom(void *p,const uint8_t *data){std::copy_n(data,8192,static_cast<Host*>(p)->rom.begin());}
}
