import { useState } from "react";
import RecordForm from "./components/records/RecordForm";
import RecordList from "./components/records/RecordList";
import GeoIpList from "./components/geoIp/GeoIpList";

function App() {
  const [tab, setTab] = useState("records");
  const [refreshKey, setRefreshKey] = useState(0);

  return (
    <div>
      <header className="app-header">
        <h1>DNS UI</h1>
        <nav className="app-tabs">
          <button className={tab === "records" ? "active" : ""} onClick={() => setTab("records")}>
            Registros
          </button>
          <button className={tab === "geoip" ? "active" : ""} onClick={() => setTab("geoip")}>
            IP → País
          </button>
        </nav>
      </header>

      <main className="app-content">
        {tab === "records" ? (
          <>
            <RecordForm onSaved={() => setRefreshKey((k) => k + 1)} />
            <RecordList key={refreshKey} />
          </>
        ) : (
          <GeoIpList />
        )}
      </main>
    </div>
  );
}

export default App;
