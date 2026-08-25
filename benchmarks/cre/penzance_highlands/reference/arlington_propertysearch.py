import re, html, urllib.parse, subprocess, json, sys
UA="Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126 Safari/537.36"
BASE="https://propertysearch.arlingtonva.us"

def curl(url, data=None):
    cmd=["curl","-sSL","-A",UA,"-b","cj.txt","-c","cj.txt","-H",f"Referer: {BASE}/Home/Search"]
    if data:
        for k,v in data: cmd += ["--data-urlencode", f"{k}={v}"]
    cmd.append(url)
    return subprocess.run(cmd,capture_output=True,text=True).stdout

def text(h):
    h=re.sub(r'<script.*?</script>','',h,flags=re.S)
    h=re.sub(r'<style.*?</style>','',h,flags=re.S)
    h=re.sub(r'<[^>]+>',' ',h)
    return re.sub(r'\s+',' ',html.unescape(h)).strip()

def search(term):
    h=curl(f"{BASE}/Home/Search",[("SearchFilters.RPCs",term),("action","Search")])
    t=text(h)
    lrsns=sorted(set(re.findall(r'lrsn=(\d+)',h)))
    i=t.find('returned')
    return lrsns, t[i:i+700] if i>0 else t[:400]

if __name__=="__main__":
    rpcs=["16033008","16033009","16033010","16033011","16033012","16033013",
          "16033014","16033016","16033017","16033018","16033021","16033022"]
    out={}
    for r in rpcs:
        l,s=search(r)
        out[r]={"lrsn":l,"summary":s}
        print(f"\n===== {r}  lrsn={l}")
        print(s[:520])
    json.dump(out,open("rpc_scan.json","w"),indent=1)

def addr(num, direction, street, stype, unit=""):
    h=curl(f"{BASE}/Home/Search",[
        ("SearchFilters.StreetNumber",num),("SearchFilters.DirectionSelected",direction),
        ("SearchFilters.StreetSelected",street),("SearchFilters.TypeSelected",stype),
        ("SearchFilters.Unit",unit),("action","Search")])
    t=text(h); i=t.find('returned')
    return sorted(set(re.findall(r'lrsn=(\d+)',h))), (t[i:i+1500] if i>0 else t[:400])

def trade(name):
    h=curl(f"{BASE}/Home/Search",[("SearchFilters.TradeName",name),("action","Search")])
    t=text(h); i=t.find('returned')
    return sorted(set(re.findall(r'lrsn=(\d+)',h))), (t[i:i+2200] if i>0 else t[:300])
