FROM grafana/grafana:10.4.2
USER root

COPY alerting /etc/grafana/provisioning/alerting/.
COPY datasources /etc/grafana/provisioning/datasources/.
COPY dashboards_yaml /etc/grafana/provisioning/dashboards/.
COPY dashboards_json /var/lib/grafana/dashboards/.

RUN chown -R grafana /etc/grafana/provisioning/ /etc/grafana/provisioning/alerting/ /etc/grafana/provisioning/dashboards/ /var/lib/grafana/dashboards/

USER grafana
WORKDIR /
ENTRYPOINT [ "/run.sh" ]
