#include "absl/time/time.h"
#include <cstdio>
#include <cstdint>
int main(int argc, char**argv){
  absl::TimeZone tz;
  if(!absl::LoadTimeZone(argv[1], &tz)){ fprintf(stderr,"LOAD_FAIL %s\n", argv[1]); return 2; }
  long long e; char line[64];
  while(fgets(line,sizeof line,stdin)){
    if(sscanf(line,"%lld",&e)!=1) continue;
    absl::Time t = absl::FromUnixSeconds(e);
    absl::TimeZone::CivilInfo ci = tz.At(t);
    printf("%lld %d %d %s\n", e, ci.offset, ci.is_dst?1:0, std::string(ci.zone_abbr).c_str());
  }
  return 0;
}
