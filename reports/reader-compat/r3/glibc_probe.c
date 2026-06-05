#include <stdio.h>
#include <stdlib.h>
#include <time.h>
#include <string.h>
int main(int argc, char**argv){
  char tz[4096]; snprintf(tz,sizeof tz, ":%s", argv[1]);
  setenv("TZ", tz, 1); tzset();
  long long e; char line[64];
  while(fgets(line,sizeof line,stdin)){
    if(sscanf(line,"%lld",&e)!=1) continue;
    time_t t=(time_t)e; struct tm tmv; localtime_r(&t,&tmv);
    printf("%lld %ld %d %s\n", e, tmv.tm_gmtoff, tmv.tm_isdst>0?1:0, tmv.tm_zone?tmv.tm_zone:"?");
  }
  return 0;
}
