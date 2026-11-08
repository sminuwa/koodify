const $ = (s, root=document) => root.querySelector(s);
const $$ = (s, root=document) => [...root.querySelectorAll(s)];
const authShell = $("#authShell"), dashboard = $("#dashboard");
const loginView = $("#loginView"), registerView = $("#registerView");
const toast = $("#toast");
$("#year").textContent = new Date().getFullYear();

function notify(message){toast.textContent=message;toast.classList.add("show");clearTimeout(notify.timer);notify.timer=setTimeout(()=>toast.classList.remove("show"),2800)}
function showView(view){
  const isRegister=view==="register";
  loginView.classList.toggle("hidden",isRegister);registerView.classList.toggle("hidden",!isRegister);
  $("#brandHeadline").innerHTML=isRegister?"A better way to<br>run your platform.":"Manage, integrate<br>and grow.";
  $("#brandDescription").textContent=isRegister?"Create your administrator profile and get a clearer view of every moving part.":"Your command centre for merchants, transactions and the services powering your business.";
}
$$("[data-view]").forEach(btn=>btn.addEventListener("click",()=>showView(btn.dataset.view)));
$$(".show-password").forEach(btn=>btn.addEventListener("click",()=>{const input=$("#"+btn.dataset.target);input.type=input.type==="password"?"text":"password";btn.textContent=input.type==="password"?"◉":"◎"}));

function getUsers(){try{return JSON.parse(localStorage.getItem("koodify_demo_users")||"[]")}catch{return []}}
function saveUsers(users){localStorage.setItem("koodify_demo_users",JSON.stringify(users))}
function setSession(user){sessionStorage.setItem("koodify_demo_session",JSON.stringify({name:user.name,email:user.email}));openDashboard(user)}
function openDashboard(user){
  authShell.classList.add("hidden");dashboard.classList.remove("hidden");
  const name=user?.name||"Admin User", initials=name.split(/\s+/).slice(0,2).map(x=>x[0]).join("").toUpperCase();
  $("#profileName").textContent=name;$("#avatarInitials").textContent=initials;$("#topAvatar").textContent=initials;$("#welcomeName").textContent="Welcome back, "+name.split(" ")[0];
  drawChart();
}
function closeDashboard(){sessionStorage.removeItem("koodify_demo_session");dashboard.classList.add("hidden");authShell.classList.remove("hidden");showView("login")}
$("#loginForm").addEventListener("submit",e=>{
  e.preventDefault();const email=$("#loginEmail").value.trim().toLowerCase(),password=$("#loginPassword").value;
  const msg=$("#loginMessage");msg.className="form-message";msg.textContent="";
  if(!email||!password){msg.textContent="Enter your email address and password.";return}
  const user=getUsers().find(u=>u.email===email&&u.password===password);
  if(user){if($("#rememberMe").checked)localStorage.setItem("koodify_demo_remember",email);setSession(user);return}
  // Demo access allows a quick preview without a backend.
  if(email==="admin@koodify.ng"&&password==="Admin123!"){setSession({name:"Koodify Admin",email});return}
  msg.textContent="We couldn't verify those details. Register a demo account or use admin@koodify.ng / Admin123!.";
});
$("#registerForm").addEventListener("submit",e=>{
  e.preventDefault();const name=$("#fullName").value.trim(),phone=$("#registerPhone").value.trim(),email=$("#registerEmail").value.trim().toLowerCase(),password=$("#registerPassword").value,confirm=$("#confirmPassword").value;
  const msg=$("#registerMessage");msg.className="form-message";msg.textContent="";
  if(!name||!email||!password||!confirm){msg.textContent="Please complete all required fields.";return}
  if(!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)){msg.textContent="Enter a valid email address.";return}
  if(password.length<8){msg.textContent="Your password must be at least 8 characters.";return}
  if(password!==confirm){msg.textContent="Your passwords do not match.";return}
  if(!$("#acceptTerms").checked){msg.textContent="Please accept the terms to continue.";return}
  const users=getUsers();if(users.some(u=>u.email===email)){msg.textContent="An account with this email already exists.";return}
  const user={name,phone,email,password};users.push(user);saveUsers(users);msg.className="form-message success";msg.textContent="Account created. Opening your dashboard…";setTimeout(()=>setSession(user),550);
});
$("#registerPassword").addEventListener("input",e=>{
  const p=e.target.value;let score=0;if(p.length>=8)score++;if(/[A-Z]/.test(p)&&/[a-z]/.test(p))score++;if(/\d/.test(p))score++;if(/[^A-Za-z0-9]/.test(p))score++;
  $("#strengthBar").style.width=(score*25)+"%";$("#strengthBar").style.background=score<2?"#ed9a45":score<4?"#e6bd45":"#20b77a";
  $("#strengthText").textContent=score<2?"Weak password":score<4?"Good — add a number and symbol":"Strong password";
});
$("#forgotBtn").addEventListener("click",()=>{const email=$("#loginEmail").value.trim();if(!email){$("#loginMessage").textContent="Enter your email address first.";return}notify("Password reset is a demo flow. Connect your email service to enable it.")});
$("#logoutBtn").addEventListener("click",closeDashboard);
$("#menuBtn").addEventListener("click",()=>$("#sidebar").classList.toggle("open"));
$$(".nav-item").forEach(btn=>btn.addEventListener("click",()=>{$$(".nav-item").forEach(x=>x.classList.remove("active"));btn.classList.add("active");$("#breadcrumbTitle").textContent=btn.dataset.section;$("#pageTitle").textContent=btn.dataset.section;$("#sidebar").classList.remove("open");if(btn.dataset.section!=="Overview")notify(btn.dataset.section+" section is a UI preview.")}));
$("#exportBtn").addEventListener("click",()=>{const csv="Reference,Merchant,Service,Date,Amount,Status\nTXN-839201,Swift Basket,Payment,Nov 07 2026,245000,Successful\nTXN-839200,Nova Goods,Transfer,Nov 07 2026,82500,Pending\nTXN-839199,Amina Mart,Virtual account,Nov 06 2026,1250000,Successful\nTXN-839198,FreshCart,Payment,Nov 06 2026,36700,Failed";const blob=new Blob([csv],{type:"text/csv"});const url=URL.createObjectURL(blob);const a=document.createElement("a");a.href=url;a.download="koodify-transactions.csv";a.click();URL.revokeObjectURL(url);notify("Report exported.")});
$("#viewAllBtn").addEventListener("click",()=>{const target=$('[data-section="Transactions"]');target.click()});
$$(".segmented button").forEach(btn=>btn.addEventListener("click",()=>{$$(".segmented button").forEach(x=>x.classList.remove("selected"));btn.classList.add("selected");$("#chartTotal").textContent=btn.dataset.chart==="volume"?"₦2,457,930,000":"1,248,320 transactions";drawChart(btn.dataset.chart)}));
function drawChart(mode="volume"){
  const canvas=$("#volumeChart");if(!canvas)return;const rect=canvas.getBoundingClientRect();const dpr=window.devicePixelRatio||1;canvas.width=Math.max(300,rect.width*dpr);canvas.height=174*dpr;
  const ctx=canvas.getContext("2d");ctx.scale(dpr,dpr);const w=canvas.width/dpr,h=174;const vals=mode==="count"?[32,48,42,65,58,86,74,100,89,112,104,132]:[52,70,64,95,82,120,112,155,143,185,170,218];
  const max=mode==="count"?150:250;const x=i=>i/(vals.length-1)*(w-8)+4,y=v=>h-8-(v/max)*(h-18);
  ctx.strokeStyle="#eff0f6";ctx.lineWidth=1;for(let i=0;i<6;i++){let yy=8+i*30;ctx.beginPath();ctx.moveTo(0,yy);ctx.lineTo(w,yy);ctx.stroke()}
  const points=vals.map((v,i)=>[x(i),y(v)]);ctx.beginPath();points.forEach(([px,py],i)=>i?ctx.lineTo(px,py):ctx.moveTo(px,py));ctx.lineTo(w,h);ctx.lineTo(0,h);ctx.closePath();const grad=ctx.createLinearGradient(0,0,0,h);grad.addColorStop(0,"rgba(117,80,238,.22)");grad.addColorStop(1,"rgba(117,80,238,0)");ctx.fillStyle=grad;ctx.fill();
  ctx.beginPath();points.forEach(([px,py],i)=>i?ctx.lineTo(px,py):ctx.moveTo(px,py));ctx.strokeStyle="#7955ef";ctx.lineWidth=2.5;ctx.lineJoin="round";ctx.lineCap="round";ctx.stroke();
  points.forEach(([px,py],i)=>{if(i===points.length-1){ctx.beginPath();ctx.arc(px,py,3.5,0,Math.PI*2);ctx.fillStyle="#7955ef";ctx.fill()}})
}
window.addEventListener("resize",()=>{if(!dashboard.classList.contains("hidden"))drawChart($(".segmented button.selected")?.dataset.chart||"volume")});
const remembered=localStorage.getItem("koodify_demo_remember");if(remembered)$("#loginEmail").value=remembered;
try{const session=JSON.parse(sessionStorage.getItem("koodify_demo_session")||"null");if(session)openDashboard(session)}catch{}
