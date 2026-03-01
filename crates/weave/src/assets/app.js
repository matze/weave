function stemFromUrl() {
    var m = location.pathname.match(/^\/(note|f)\/(.+)/);
    return m ? decodeURIComponent(m[2]) : null;
}

function syncView(scroll) {
    var stem = stemFromUrl();
    document.body.toggleAttribute('data-note', !!stem);
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
        document.body.dataset.note = '1';
    }
}

function showList() {
    delete document.body.dataset.note;
}

function goBack() {
    if (history.length > 1) history.back();
    else location.href = '/';
}

function toggleFocus() {
    if (window.innerWidth < 768) return;
    var prose = document.querySelector('#note-content .prose');
    if (document.body.classList.contains('focus-mode')) {
        if (prose) {
            prose.style.maxWidth = prose.offsetWidth + 'px';
            prose.style.marginLeft = getComputedStyle(prose).marginLeft;
            prose.style.marginRight = getComputedStyle(prose).marginRight;
        }
        document.body.classList.remove('focus-mode');
        if (prose) {
            requestAnimationFrame(function() {
                prose.style.marginLeft = '';
                prose.style.marginRight = '';
            });
            document.getElementById('sidebar').addEventListener('transitionend', function handler(e) {
                if (e.propertyName === 'width') {
                    prose.style.maxWidth = '';
                    document.getElementById('sidebar').removeEventListener('transitionend', handler);
                }
            });
        }
    } else {
        if (prose) {
            document.body.style.setProperty('--focus-prose-width', prose.offsetWidth + 'px');
        }
        document.body.classList.add('focus-mode');
    }
}

document.addEventListener('keydown', function(e) {
    if (e.key === 'Escape') {
        if (document.body.classList.contains('focus-mode')) {
            toggleFocus();
            return;
        }
        if (document.activeElement === document.getElementById('filter-input')) {
            document.activeElement.blur();
            return;
        }
    }
    var t = document.activeElement.tagName;
    if (t === 'INPUT' || t === 'TEXTAREA' || t === 'SELECT') return;
    if (e.key === 'e') {
        var btn = document.querySelector('[aria-label="Edit note"]');
        if (btn) btn.click();
    } else if (e.key === 'f') {
        e.preventDefault();
        toggleFocus();
    } else if (e.key === 's') {
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

document.addEventListener('DOMContentLoaded', function() {
    syncView(true);
    var filterInput = document.getElementById('filter-input');
    var filterClear = document.getElementById('filter-clear');
    filterInput.addEventListener('input', function() {
        filterClear.classList.toggle('hidden', !this.value);
    });
    filterClear.addEventListener('click', function() {
        filterInput.value = '';
        htmx.ajax('POST', '/f/search', {target: '#search-list', values: {query: ''}});
        filterClear.classList.add('hidden');
    });
});

window.addEventListener('popstate', function() {
    var stem = stemFromUrl();
    if (stem) {
        htmx.ajax('GET', '/f/' + encodeURIComponent(stem), {target: '#note-content'});
    }
    syncView(true);
});

document.addEventListener('htmx:historyRestore', function() {
    syncView(true);
    var filterInput = document.getElementById('filter-input');
    var filterClear = document.getElementById('filter-clear');
    if (filterInput && filterClear) {
        filterClear.classList.toggle('hidden', !filterInput.value);
        htmx.ajax('POST', '/f/search', {target: '#search-list', values: {query: filterInput.value}});
    }
});

document.addEventListener('htmx:afterSettle', function(e) {
    if (e.detail.target.id === 'note-content') {
        syncView(false);
    }
});

function showNoteError(message) {
    var nc = document.getElementById('note-content');
    if (!nc) return;
    nc.innerHTML =
        '<div class="p-4 border-b border-gray-200 dark:border-gray-700 flex-shrink-0">' +
          '<div class="flex items-center gap-3">' +
            '<button class="md:hidden p-1 -ml-1 rounded hover:bg-gray-100 dark:hover:bg-gray-700" onclick="goBack()" aria-label="Back to notes">' +
              '<svg class="w-6 h-6" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24"><path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7"/></svg>' +
            '</button>' +
            '<span class="invisible font-black text-xl flex-grow">\u00a0</span>' +
          '</div>' +
        '</div>' +
        '<div class="flex-grow flex items-center justify-center">' +
          '<div class="flex flex-col items-center justify-center p-8 text-center">' +
            '<h2 class="text-xl font-bold text-gray-400 dark:text-gray-500">' + message + '</h2>' +
          '</div>' +
        '</div>';
}

document.addEventListener('htmx:sendError', function(e) {
    if (e.detail.target && e.detail.target.id === 'note-content') {
        showNoteError('note could not be loaded');
    }
});

document.addEventListener('htmx:responseError', function(e) {
    if (e.detail.target && e.detail.target.id === 'note-content') {
        showNoteError('note could not be loaded');
    }
});
