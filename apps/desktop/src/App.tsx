import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
export default function App(){const [info,setInfo]=useState('Povezovanje z lokalnim jedrom …');useEffect(()=>{invoke<{name:string;version:string}>('get_app_info').then(x=>setInfo(x.name+' '+x.version)).catch(()=>setInfo('Lokalno jedro ni na voljo. Odpri namizno aplikacijo.'));},[]);return <main><small>PREFLIGHT / LOKALNA DIAGNOSTIKA</small><h1>Pripravljeni na delavnico.</h1><p>Rezultati ostanejo na tvojem računalniku.</p><p role="status">{info}</p><p>Preverjanje okolja je v pripravi.</p></main>}
