// Injects the on-canvas edit/selection overlay into the design HTML.
// Ported verbatim from ui.html injectEditSupport() so the DOM inside the
// iframe behaves identically (data-ov-sel selection, drag/drop, edit toolbar).

// Strip any previous injection so we don't accumulate style/script blocks
// each time the iframe re-renders (the iframe posts its full outerHTML on
// edit; re-wrapping without stripping would double the injected code and
// leave multiple toolbars stacked in the DOM).
export function stripEditArtifacts(html) {
  if (!html) return '';
  return html
    .replace(/<style\s+id="__ov_style"[^>]*>[\s\S]*?<\/style>\s*/gi, '')
    .replace(/<script\s+id="__edit_support"[^>]*>[\s\S]*?<\/script>\s*/gi, '')
    .replace(/<div\s+id="__ov_(?:tag|tb|drop|ghost)"[^>]*>[\s\S]*?<\/div>\s*/gi, '')
    .replace(/\s(?:data-ov-sel|data-ov-hover|data-ov-dragging|data-ov-drop-into)(?:="[^"]*")?/gi, '')
    .replace(/\scontenteditable(?:="[^"]*")?/gi, '');
}

export function injectEditSupport(rawHtml) {
  const html = stripEditArtifacts(rawHtml);
  const CT = '<' + '/script>';
  const CS = '<' + '/style>';

  const style =
    '<style id="__ov_style">' +
    '[data-ov-hover]{outline:1px dashed rgba(184,64,46,0.55) !important;outline-offset:1px;}' +
    '[data-ov-sel]{outline:2px solid #b8402e !important;outline-offset:1px;box-shadow:0 0 0 3px rgba(184,64,46,0.14);}' +
    '[data-ov-dragging]{opacity:0.35 !important;}' +
    '[data-ov-drop-into]{outline:2px dashed #b8402e !important;outline-offset:-2px;background:rgba(184,64,46,0.06) !important;}' +
    '#__ov_ghost{position:fixed;pointer-events:none;z-index:2147483646;opacity:0.6;transform:rotate(-1deg);box-shadow:0 12px 40px rgba(0,0,0,0.25);border:1px solid #b8402e;background:white;overflow:hidden;}' +
    '#__ov_tag{position:fixed;background:#b8402e;color:#fff;padding:2px 7px;' +
      'font-family:"JetBrains Mono",ui-monospace,Menlo,monospace;font-size:10px;' +
      'letter-spacing:0.04em;border-radius:2px;z-index:2147483647;pointer-events:none;display:none;}' +
    '#__ov_tb{position:fixed;background:#1c1a17;color:#f4ede0;border:1px solid rgba(184,64,46,0.55);' +
      'border-radius:4px;padding:3px;display:none;z-index:2147483647;' +
      'box-shadow:0 6px 22px rgba(0,0,0,0.28);font-family:system-ui,sans-serif;gap:1px;}' +
    '#__ov_tb button{background:transparent;border:none;color:inherit;padding:5px 8px;' +
      'font-size:12px;line-height:1;border-radius:3px;cursor:pointer;font-family:inherit;}' +
    '#__ov_tb button:hover{background:rgba(184,64,46,0.28);}' +
    '#__ov_tb button[data-a="del"]{color:#e57856;}' +
    '#__ov_tb button[data-a="drag"]{cursor:grab;}' +
    '#__ov_drop{position:fixed;height:3px;background:#b8402e;pointer-events:none;' +
      'z-index:2147483647;border-radius:2px;box-shadow:0 0 10px rgba(184,64,46,0.55);display:none;}' +
    CS;

  const script =
    '<script id="__edit_support">' +
    '(function(){' +
    'var IGNORE=/^(HTML|HEAD|BODY|SCRIPT|STYLE|META|LINK|TITLE|IFRAME|NOSCRIPT)$/;' +
    'var TEXT_TAGS=/^(H1|H2|H3|H4|H5|H6|P|LI|TD|TH|SPAN|BUTTON|A|LABEL|EM|STRONG|SMALL|B|I|U|BLOCKQUOTE|CAPTION|FIGCAPTION|DT|DD|SUMMARY|CODE)$/;' +
    'var selected=null,hovered=null,editing=false,dragging=false,dragEl=null,dropTarget=null,dropMode="",playMode=false,potentialDrag=null,ghost=null;' +
    'var CONTAINERS=/^(DIV|SECTION|HEADER|FOOTER|MAIN|ARTICLE|ASIDE|NAV|UL|OL|FORM|FIGURE|BODY)$/;' +

    'var tag=document.createElement("div");tag.id="__ov_tag";document.documentElement.appendChild(tag);' +
    'var tb=document.createElement("div");tb.id="__ov_tb";' +
    'tb.innerHTML=' +
      '\'<button data-a="drag" title="Drag to reorder">\\u2630</button>\'+' +
      '\'<button data-a="up" title="Move up">\\u2191</button>\'+' +
      '\'<button data-a="dn" title="Move down">\\u2193</button>\'+' +
      '\'<button data-a="dup" title="Duplicate">\\u29c9</button>\'+' +
      '\'<button data-a="edit" title="Edit text">\\u270e</button>\'+' +
      '\'<button data-a="parent" title="Select parent">\\u2196</button>\'+' +
      '\'<button data-a="del" title="Delete">\\u00d7</button>\';' +
    'document.documentElement.appendChild(tb);' +
    'var drop=document.createElement("div");drop.id="__ov_drop";document.documentElement.appendChild(drop);' +

    'function isEditable(el){' +
      'if(!el||el.nodeType!==1)return false;' +
      'if(IGNORE.test(el.tagName))return false;' +
      'if(el.id&&el.id.indexOf("__ov")===0)return false;' +
      'if(el.id==="__edit_support"||el.id==="__ov_style")return false;' +
      'if(el.closest&&el.closest("#__ov_tb,#__ov_tag,#__ov_drop"))return false;' +
      'return true;' +
    '}' +

    'function elAt(x,y){' +
      'tb.style.pointerEvents="none";tag.style.pointerEvents="none";drop.style.pointerEvents="none";' +
      'var el=document.elementFromPoint(x,y);' +
      'tb.style.pointerEvents="";' +
      'return el;' +
    '}' +

    'function selectorFor(el){' +
      'if(!el)return "";' +
      'if(el.id&&!/^__ov/.test(el.id))return "#"+el.id;' +
      'var s=el.tagName.toLowerCase();' +
      'if(typeof el.className==="string"){' +
        'var cls=el.className.trim().split(/\\s+/).filter(function(c){return c&&!/^__ov/.test(c);});' +
        'if(cls.length)s+="."+cls.slice(0,2).join(".");' +
      '}' +
      'return s;' +
    '}' +

    'function anchor(){' +
      'if(selected&&!selected.isConnected){selected=null;}' +
      'document.querySelectorAll("[data-ov-hover]").forEach(function(e){e.removeAttribute("data-ov-hover");});' +
      'document.querySelectorAll("[data-ov-sel]").forEach(function(e){e.removeAttribute("data-ov-sel");});' +
      'if(hovered&&hovered!==selected&&isEditable(hovered))hovered.setAttribute("data-ov-hover","");' +
      'if(selected)selected.setAttribute("data-ov-sel","");' +
      'if(!selected){tb.style.display="none";tag.style.display="none";return;}' +
      'var r=selected.getBoundingClientRect();' +
      'tag.style.display="block";' +
      'tag.style.left=Math.max(2,r.left)+"px";' +
      'tag.style.top=Math.max(2,r.top-18)+"px";' +
      'tag.textContent="<"+selectorFor(selected)+">";' +
      'tb.style.display="flex";' +
      'var tbr=tb.getBoundingClientRect();' +
      'var top=r.top-tbr.height-6;' +
      'if(top<4)top=r.bottom+6;' +
      'var left=r.left;' +
      'if(left+tbr.width>window.innerWidth-4)left=window.innerWidth-tbr.width-4;' +
      'tb.style.top=Math.max(4,top)+"px";' +
      'tb.style.left=Math.max(4,left)+"px";' +
    '}' +

    'function notifyEdit(){' +
      'try{window.parent.postMessage({type:"html_edited",html:"<!DOCTYPE html>"+document.documentElement.outerHTML},"*");}catch(e){}' +
    '}' +
    'function notifySel(){' +
      'try{window.parent.postMessage({type:"selection",selector:selectorFor(selected),tag:selected?selected.tagName.toLowerCase():""},"*");}catch(e){}' +
    '}' +

    'function select(el){' +
      'if(editing)endEdit();' +
      'selected=el;' +
      'anchor();notifySel();' +
    '}' +

    'function beginEdit(){' +
      'if(!selected)return;' +
      'editing=true;' +
      'selected.setAttribute("contenteditable","true");' +
      'selected.focus();' +
      'var r=document.createRange();r.selectNodeContents(selected);' +
      'var s=window.getSelection();s.removeAllRanges();s.addRange(r);' +
    '}' +
    'function endEdit(){' +
      'if(!editing||!selected){editing=false;return;}' +
      'selected.removeAttribute("contenteditable");' +
      'editing=false;notifyEdit();anchor();' +
    '}' +

    'function doAction(a){' +
      'if(!selected)return;' +
      'if(a==="up"){' +
        'var p=selected.previousElementSibling;' +
        'if(p&&isEditable(p))selected.parentNode.insertBefore(selected,p);' +
      '}else if(a==="dn"){' +
        'var n=selected.nextElementSibling;' +
        'if(n&&isEditable(n))selected.parentNode.insertBefore(n,selected);' +
      '}else if(a==="dup"){' +
        'var c=selected.cloneNode(true);' +
        'selected.parentNode.insertBefore(c,selected.nextSibling);' +
      '}else if(a==="del"){' +
        'var rm=selected;selected=null;rm.parentNode.removeChild(rm);' +
      '}else if(a==="edit"){' +
        'beginEdit();return;' +
      '}else if(a==="parent"){' +
        'var pr=selected.parentElement;' +
        'if(pr&&isEditable(pr))selected=pr;' +
      '}' +
      'anchor();notifyEdit();notifySel();' +
    '}' +

    'tb.querySelectorAll("button").forEach(function(b){' +
      'b.addEventListener("mousedown",function(e){e.preventDefault();e.stopPropagation();});' +
      'b.addEventListener("click",function(e){' +
        'e.preventDefault();e.stopPropagation();' +
        'if(b.dataset.a!=="drag")doAction(b.dataset.a);' +
      '});' +
    '});' +

    'var dragBtn=tb.querySelector(\'[data-a="drag"]\');' +
    'dragBtn.addEventListener("mousedown",function(e){' +
      'if(!selected)return;' +
      'e.preventDefault();e.stopPropagation();' +
      'startDrag(selected,e.clientX,e.clientY);' +
    '});' +

    'document.addEventListener("mousedown",function(e){' +
      'if(playMode||editing||dragging)return;' +
      'if(e.target.closest&&e.target.closest("#__ov_tb"))return;' +
      'if(!selected||!selected.contains(e.target))return;' +
      'potentialDrag={x:e.clientX,y:e.clientY,el:selected};' +
    '},true);' +

    'function startDrag(el,x,y){' +
      'dragging=true;dragEl=el;potentialDrag=null;' +
      'dragEl.setAttribute("data-ov-dragging","");' +
      'var r=dragEl.getBoundingClientRect();' +
      'ghost=dragEl.cloneNode(true);ghost.id="__ov_ghost";' +
      'ghost.removeAttribute("data-ov-sel");' +
      'ghost.removeAttribute("data-ov-hover");' +
      'ghost.removeAttribute("data-ov-dragging");' +
      'ghost.style.top=r.top+"px";ghost.style.left=r.left+"px";' +
      'ghost.style.width=r.width+"px";ghost.style.height=r.height+"px";' +
      'document.documentElement.appendChild(ghost);' +
      'document.body.style.userSelect="none";' +
      'document.body.style.cursor="grabbing";' +
      'tb.style.display="none";tag.style.display="none";' +
      'document.querySelectorAll("[data-ov-hover]").forEach(function(x){x.removeAttribute("data-ov-hover");});' +
    '}' +

    'function clearDropUI(){' +
      'drop.style.display="none";' +
      'document.querySelectorAll("[data-ov-drop-into]").forEach(function(x){x.removeAttribute("data-ov-drop-into");});' +
    '}' +

    'document.addEventListener("mousemove",function(e){' +
      'if(playMode)return;' +
      'if(potentialDrag&&!dragging){' +
        'var dx=e.clientX-potentialDrag.x,dy=e.clientY-potentialDrag.y;' +
        'if(dx*dx+dy*dy>25){startDrag(potentialDrag.el,e.clientX,e.clientY);}' +
        'else return;' +
      '}' +
      'if(dragging){' +
        'if(ghost){ghost.style.left=(e.clientX-ghost.offsetWidth/2)+"px";ghost.style.top=(e.clientY-20)+"px";}' +
        'var el=elAt(e.clientX,e.clientY);' +
        'if(!el||!isEditable(el)||el===dragEl||dragEl.contains(el)){clearDropUI();dropTarget=null;dropMode="";return;}' +
        'var r=el.getBoundingClientRect();' +
        'var yIn=e.clientY-r.top,h=r.height;' +
        'var isContainer=CONTAINERS.test(el.tagName);' +
        'var mode;' +
        'if(isContainer&&yIn>h*0.25&&yIn<h*0.75){mode="into";}' +
        'else if(yIn<h*0.5){mode="before";}' +
        'else{mode="after";}' +
        'clearDropUI();' +
        'dropTarget=el;dropMode=mode;' +
        'if(mode==="into"){el.setAttribute("data-ov-drop-into","");}' +
        'else{' +
          'drop.style.display="block";' +
          'drop.style.left=r.left+"px";drop.style.width=r.width+"px";' +
          'drop.style.top=(mode==="before"?r.top-2:r.bottom-1)+"px";' +
        '}' +
        'return;' +
      '}' +
      'if(editing)return;' +
      'var hv=elAt(e.clientX,e.clientY);' +
      'if(!hv||!isEditable(hv)){hovered=null;anchor();return;}' +
      'hovered=hv;anchor();' +
    '});' +

    'document.addEventListener("mouseup",function(){' +
      'potentialDrag=null;' +
      'if(!dragging)return;' +
      'dragging=false;' +
      'document.body.style.userSelect="";' +
      'document.body.style.cursor="";' +
      'if(ghost){ghost.remove();ghost=null;}' +
      'if(dragEl)dragEl.removeAttribute("data-ov-dragging");' +
      'if(dropTarget&&dragEl&&!dragEl.contains(dropTarget)){' +
        'if(dropMode==="into"){dropTarget.appendChild(dragEl);}' +
        'else if(dropTarget.parentNode){' +
          'dropTarget.parentNode.insertBefore(dragEl,dropMode==="before"?dropTarget:dropTarget.nextSibling);' +
        '}' +
        'selected=dragEl;' +
      '}' +
      'clearDropUI();' +
      'dropTarget=null;dropMode="";dragEl=null;' +
      'anchor();notifyEdit();notifySel();' +
    '});' +

    'document.addEventListener("click",function(e){' +
      'if(playMode)return;' +
      'if(e.target.closest&&e.target.closest("#__ov_tb"))return;' +
      'if(editing&&selected&&selected.contains(e.target))return;' +
      'if(editing)endEdit();' +
      'var el=e.target;' +
      'if(!isEditable(el)){select(null);return;}' +
      'e.preventDefault();e.stopPropagation();' +
      'select(el);' +
    '},true);' +

    'document.addEventListener("dblclick",function(e){' +
      'if(playMode)return;' +
      'var el=e.target;' +
      'if(!isEditable(el)||!TEXT_TAGS.test(el.tagName))return;' +
      'e.preventDefault();e.stopPropagation();' +
      'select(el);beginEdit();' +
    '},true);' +

    'document.addEventListener("keydown",function(e){' +
      'if(e.key==="Escape"){' +
        'if(editing){endEdit();return;}' +
        'if(selected){select(null);}return;' +
      '}' +
      'if((e.key==="Delete"||e.key==="Backspace")&&selected&&!editing){' +
        'var t=e.target;' +
        'if(t&&(t.isContentEditable||t.tagName==="INPUT"||t.tagName==="TEXTAREA"))return;' +
        'e.preventDefault();doAction("del");' +
      '}' +
      'if((e.metaKey||e.ctrlKey)&&(e.key==="z"||e.key==="Z")){' +
        'var ae=document.activeElement;' +
        'if(ae&&(ae.isContentEditable||ae.tagName==="INPUT"||ae.tagName==="TEXTAREA"))return;' +
        'e.preventDefault();' +
        'try{window.parent.postMessage({type:"canvas_undo_request",redo:!!e.shiftKey},"*");}catch(_){}' +
      '}' +
    '});' +

    'document.addEventListener("input",function(){if(editing)notifyEdit();});' +

    'window.addEventListener("scroll",anchor,true);' +
    'window.addEventListener("resize",anchor);' +
    'var mo=new MutationObserver(function(){anchor();});' +
    'mo.observe(document.body,{childList:true,subtree:true});' +

    'window.addEventListener("message",function(e){' +
      'if(!e.data)return;' +
      'if(e.data.cmd==="clearSelection"){select(null);}' +
      'else if(e.data.cmd==="setPlayMode"){' +
        'playMode=!!e.data.value;' +
        'if(playMode){' +
          'select(null);' +
          'document.querySelectorAll("[data-ov-hover]").forEach(function(el){el.removeAttribute("data-ov-hover");});' +
          'hovered=null;tb.style.display="none";tag.style.display="none";' +
        '}' +
      '}' +
    '});' +

    'try{window.parent.postMessage({type:"edit_ready"},"*");}catch(e){}' +
    '})();' + CT;

  const combined = style + script;
  return html.includes('</body>') ? html.replace('</body>', combined + '\n</body>') : html + combined;
}

// Prototype-mode overlay (Prototype export button).
export function buildPrototypeHTML(html) {
  const closeTag = '<' + '/script>';
  const overlay =
    '<style id="__ps">' +
    '#__pbar{position:fixed;bottom:20px;left:50%;transform:translateX(-50%);z-index:999999;' +
    'background:#1c1a17;color:#f4ede0;border-radius:6px;padding:10px 18px;' +
    'display:flex;align-items:center;gap:12px;box-shadow:0 8px 40px rgba(0,0,0,.4);' +
    'font-family:system-ui;font-size:12px;border:1px solid rgba(184,64,46,.4);}' +
    '#__pbar button{background:rgba(184,64,46,.15);border:1px solid rgba(184,64,46,.5);' +
    'color:#d76146;border-radius:4px;padding:4px 10px;font-size:11px;cursor:pointer;' +
    'letter-spacing:0.06em;text-transform:uppercase;font-weight:500;}' +
    '#__pbar button:hover{background:rgba(184,64,46,.3);}' +
    '.ph{outline:2px solid rgba(184,64,46,.7)!important;outline-offset:2px;}' +
    '<' + '/style>' +
    '<div id="__pbar">' +
    '<span style="opacity:.6;letter-spacing:0.14em;text-transform:uppercase;font-size:10px">Prototype</span>' +
    '<span id="__pinfo">Ready</span>' +
    '<button onclick="__phi()">Show interactions</button>' +
    '<button onclick="__pclose()" style="background:none;border:none;color:#f4ede0;opacity:.5;font-size:16px;padding:0 2px;cursor:pointer">×</button>' +
    '</div>' +
    '<script id="__ps2">' +
    '(function(){' +
    'var hl=false;' +
    'window.__phi=function(){' +
    '  hl=!hl;' +
    '  var els=document.querySelectorAll("button,a,[role=button],input[type=submit]");' +
    '  els.forEach(function(el,i){' +
    '    if(hl)el.classList.add("ph"); else el.classList.remove("ph");' +
    '  });' +
    '  document.getElementById("__pinfo").textContent=hl?els.length+" interactive elements":"Ready";' +
    '  document.querySelector("#__pbar button").textContent=hl?"Hide interactions":"Show interactions";' +
    '};' +
    'window.__pclose=function(){document.getElementById("__pbar").remove();};' +
    '})();' +
    closeTag;

  return html.includes('</body>') ? html.replace('</body>', overlay + '\n</body>') : html + overlay;
}
