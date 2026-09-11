
(function () {
        var LOGURL = 'https://sp1.baidu.com/5b1ZeDe5KgQFm2e88IuM_a/cm.gif';
        var params = 'type=wwwerror&terminal=www';
        var img = new Image();
        img.src = LOGURL + '?' + params;
    })();
    (function () {
        if(window.recommend && window.recommend.query && window.recommend.query.length > 0) {
            var recommendWrapper = document.createElement('div');
            var recommendHtml = '<div class="cr-content"><div class="opr-toplist-title">' + window.recommend.title + '</div><table class="opr-toplist-table"><thead><tr><th>排名</th></tr></thead>';
            var queryArray = window.recommend.query;
            var itemUrl = '';
            for(var i = 1; i < (queryArray.length+1); i++) {
                itemUrl = '//www.baidu.com/s?word=' + queryArray[i-1].word + '&sa=' + queryArray[i-1].sa;
                if (i < 4) {
                    recommendHtml += '<tr><td><span class="opr-index-hot' + i + ' opr-index-item">' + i + '</span><a target="_blank" href="' + itemUrl +'" class="opr-item-text">' + queryArray[i-1].word + '</a></td></tr>';
                } else {
                    recommendHtml += '<tr><td><span class="opr-index-item">' + i + '</span><a target="_blank" href="' + itemUrl +'" class="opr-item-text">' + queryArray[i-1].word + '</a></td></tr>';
                }
            }
            recommendHtml += '</tbody></table></div>';
            recommendWrapper.setAttribute('id', 'content_right');
            document.getElementById('wrapper_wrapper').insertBefore(recommendWrapper, document.getElementById('content_left'));
            var recommend = document.getElementById('content_right');
            recommend.innerHTML = recommendHtml;
        }
})();
(function(){
    var bds = {
        util: {}
    };
    var c = document.getElementById('kw').parentNode;

    bds.util.getWinWidth = function(){
        return window.document.documentElement.clientWidth;
    };

    bds.util.setFormWidth = function(){
        var width = bds.util.getWinWidth();
        if(width < 1217)    {bds.util.setClass(c, 'ip_short', 'add')}
        else                {bds.util.setClass(c, 'ip_short', 'remove')};
    };

    bds.util.setClass = function(obj, class_name, set) {
        var ori_class = obj.className,
            has_class_p = -1,
            ori_class_arr = [],
            new_class = '';

        if(ori_class.length) ori_class_arr = ori_class.split(' ');

        for( i in ori_class_arr) {
            if(ori_class_arr[i] == class_name) has_class_p = i;
        }

        if( set == 'remove' && has_class_p >= 0) {
            ori_class_arr.splice(has_class_p, 1);
            new_class = ori_class_arr.join(' ');
            obj.className = new_class;
        } else if( set == 'add' && has_class_p < 0) {
            ori_class_arr.push(class_name);
            new_class = ori_class_arr.join(' ');
            obj.className = new_class;
        }
    }
    bds.util.setFormWidth();

    if (typeof document.addEventListener != "undefined") {
        window.addEventListener('resize', bds.util.setFormWidth, false);
        document.getElementById('kw').addEventListener('focus', function(){bds.util.setClass(c,'iptfocus', 'add');}, false);
        document.getElementById('kw').addEventListener('blur', function(){bds.util.setClass(c,'iptfocus', 'remove');}, false);
    } else {
        window.attachEvent('onresize', bds.util.setFormWidth, false);
        document.getElementById('kw').attachEvent('onfocus', function(){bds.util.setClass(c,'iptfocus', 'add');}, false);
        document.getElementById('kw').attachEvent('onblur', function(){bds.util.setClass(c,'iptfocus', 'remove');}, false);
    } 

})();


