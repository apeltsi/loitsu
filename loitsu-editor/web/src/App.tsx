import './App.css';
import FileExplorer from './components/FileExplorer';
import Hierarchy from './components/Hierarchy';
import Inspector from './components/Inspector';
import TopBar from './components/TopBar';
function App() {
    return (
    <>
        <TopBar/>
        <div class="panel-shadow">
            <div class="primary">
                <Inspector/>
                <FileExplorer/>
                <Hierarchy/>
            </div>
        </div>
    </>
    )
}

export default App;
