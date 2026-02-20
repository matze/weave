function stemFromUrl() {
    var m = location.pathname.match(/^\/(note|f)\/(.+)/);
    return m ? decodeURIComponent(m[2]) : null;
}

function syncView(scroll) {
    var stem = stemFromUrl();
    document.getElementById('sidebar').classList.toggle('mobile-hidden', !!stem);
    document.getElementById('note-content').classList.toggle('mobile-visible', !!stem);
    document.querySelectorAll('.note-item.active-note').forEach(function(el) {
        el.classList.remove('active-note');
    });
    if (stem) {
        var el = document.querySelector('.note-item[data-stem="' + CSS.escape(stem) + '"]');
        if (el) {
            el.classList.add('active-note');
            if (scroll) el.scrollIntoView({block: 'nearest'});
        }
    }
    var h2 = document.querySelector('#note-content h2');
    document.title = h2 ? h2.textContent + ' \u{2013} weave' : 'weave';
}

function showNote(e) {
    if (e && e.currentTarget) {
        document.querySelectorAll('.note-item.active-note').forEach(function(el) {
            el.classList.remove('active-note');
        });
        e.currentTarget.classList.add('active-note');
        e.currentTarget.style.position = 'relative';
    }
}

function showList() {
    document.getElementById('sidebar').classList.remove('mobile-hidden');
    document.getElementById('note-content').classList.remove('mobile-visible');
}

var hasAppHistory = location.pathname === '/';

document.addEventListener('htmx:pushedIntoHistory', function() { hasAppHistory = true; });

function goBack() {
    if (hasAppHistory) {
        history.back();
    } else {
        location.href = '/';
    }
}

document.addEventListener('keydown', function(e) {
    if (e.key === 'Escape' && document.activeElement === document.getElementById('filter-input')) {
        document.activeElement.blur();
        return;
    }
    var t = document.activeElement.tagName;
    if (t === 'INPUT' || t === 'TEXTAREA' || t === 'SELECT') return;
    if (e.key === 'e') {
        var btn = document.querySelector('[aria-label="Edit note"]');
        if (btn) btn.click();
    } else if (e.key === 'f') {
        e.preventDefault();
        document.getElementById('filter-input').focus();
    } else if (e.key === 'j' || e.key === 'k') {
        var items = Array.from(document.querySelectorAll('#search-list .note-item'));
        if (!items.length) return;
        var active = document.querySelector('.note-item.active-note');
        var idx = active ? items.indexOf(active) : -1;
        var next = e.key === 'j' ? idx + 1 : idx - 1;
        if (next < 0 || next >= items.length) return;
        items[next].click();
        items[next].scrollIntoView({block: 'nearest'});
    }
});

document.addEventListener('DOMContentLoaded', function() { syncView(true); });

window.addEventListener('popstate', function() { syncView(true); });

document.addEventListener('htmx:beforeHistorySave', function() {
    // Only sync mobile classes for the snapshot; skip active-note
    // to avoid re-marking the old note (showNote already set it).
    var stem = stemFromUrl();
    document.getElementById('sidebar').classList.toggle('mobile-hidden', !!stem);
    document.getElementById('note-content').classList.toggle('mobile-visible', !!stem);
});

document.addEventListener('htmx:historyRestore', function() {
    syncView(true);
});

document.addEventListener('htmx:afterSettle', function(e) {
    if (e.detail.target.id === 'note-content') {
        syncView(false);
    }
});
