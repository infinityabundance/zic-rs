package main
import ("bufio";"fmt";"os";"strconv";"time")
func main(){
  data,err := os.ReadFile(os.Args[1])
  if err!=nil { fmt.Fprintln(os.Stderr,"READ_FAIL",err); os.Exit(2) }
  loc,err := time.LoadLocationFromTZData(os.Args[2], data)
  if err!=nil { fmt.Fprintln(os.Stderr,"LOAD_FAIL",err); os.Exit(2) }
  sc := bufio.NewScanner(os.Stdin)
  for sc.Scan(){
    e,err := strconv.ParseInt(sc.Text(),10,64); if err!=nil { continue }
    t := time.Unix(e,0).In(loc)
    name,off := t.Zone()
    // Go exposes offset+abbr, not is_dst -> emit -1 for isdst
    fmt.Printf("%d %d -1 %s\n", e, off, name)
  }
}
